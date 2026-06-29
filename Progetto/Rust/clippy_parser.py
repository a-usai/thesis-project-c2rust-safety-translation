#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
clippy_taxonomy_parser.py
=========================
Analizza l'output JSON di `cargo clippy --message-format=json` e mappa
ogni warning nelle 18 categorie ufficiali di Tadesse et al.

    Internal Quality: Convention violation | Documentation issues |
                      Inflexible code | Misleading code | Non-idiomatic code |
                      Non-production code | Readability issues | Redundant code

    External Quality: Arithmetic issues | Attribute issues |
                      Compatibility issues | Error handling issues |
                      Logical issues | Memory safety | Performance |
                      Runtime Panic risks | Thread safety | Type safety

Utilizzo:
    cargo clippy --message-format=json 2>&1 | tee clippy_results.json
    python3 clippy_taxonomy_parser.py

    # oppure, per testare senza clippy_results.json:
    python3 clippy_taxonomy_parser.py --demo

Output:
    - Riepilogo a terminale
    - clippy_taxonomy_report.html  → report dettagliato per violazione
    - clippy_taxonomy_table.html   → tabella riepilogativa (tesi)
"""

import json
import sys
import argparse
import html as html_lib
from collections import defaultdict
from pathlib import Path
from datetime import datetime

# ---------------------------------------------------------------------------
# CONFIGURAZIONE
# ---------------------------------------------------------------------------

INPUT_FILE   = "clippy_results.json"
OUTPUT_FILE  = "clippy_taxonomy_report.html"
SUMMARY_FILE = "clippy_taxonomy_table.html"

# ---------------------------------------------------------------------------
# TASSONOMIA UFFICIALE — Tadesse et al.
# Codici: I1-I8 = Internal Quality, E1-E10 = External Quality
# ---------------------------------------------------------------------------

# Ordine display: prima Internal, poi External (come nel paper)
INTERNAL_CODES = ["I1", "I2", "I3", "I4", "I5", "I6", "I7", "I8"]
EXTERNAL_CODES = ["E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9", "E10"]

TAXONOMY: dict[str, str] = {
    # ── Internal Quality ──────────────────────────────────────────────────
    "I1": "Convention violation",
    "I2": "Documentation issues",
    "I3": "Inflexible code",
    "I4": "Misleading code",
    "I5": "Non-idiomatic code",
    "I6": "Non-production code",
    "I7": "Readability issues",
    "I8": "Redundant code",
    # ── External Quality ──────────────────────────────────────────────────
    "E1":  "Arithmetic issues",
    "E2":  "Attribute issues",
    "E3":  "Compatibility issues",
    "E4":  "Error handling issues",
    "E5":  "Logical issues",
    "E6":  "Memory safety",
    "E7":  "Performance",
    "E8":  "Runtime Panic risks",
    "E9":  "Thread safety",
    "E10": "Type safety",
    # ── Fallback ──────────────────────────────────────────────────────────
    "UNMAPPED": "Unmapped / Other",
}

# ---------------------------------------------------------------------------
# DESCRIZIONI (dalla tabella del paper, tradotte in italiano)
# ---------------------------------------------------------------------------

TAXONOMY_DESCRIPTIONS: dict[str, str] = {
    "I1": ("Codice che viola le convenzioni comuni di Rust per nomi e "
           "progettazione (snake_case, CamelCase, SCREAMING_SNAKE_CASE, "
           "ripetizione del nome del modulo)."),
    "I2": ("Problemi nei commenti o nella documentazione che riducono la "
           "comprensibilità o la manutenibilità; assenza di sezione #Safety "
           "nelle funzioni unsafe."),
    "I3": ("Codice che utilizza tipi eccessivamente specifici, limitando "
           "la riusabilità e la flessibilità dell'API."),
    "I4": ("Codice che porta il lettore a credere che faccia qualcosa di "
           "diverso da ciò che fa effettivamente (formattazione ingannevole, "
           "confronti ridondanti, cloni non necessari)."),
    "I5": ("Codice che non segue le convenzioni, i pattern o le best practice "
           "di Rust: return superflui, pattern let ref mut, iteratori "
           "non idiomatici, operatore ? mancante."),
    "I6": ("Codice pensato per debug o placeholder che non dovrebbe comparire "
           "in produzione (todo!, unimplemented!, dbg!, print!)."),
    "I7": ("Codice che rende più difficile per il lettore comprendere "
           "l'intenzione dello sviluppatore: funzioni troppo lunghe, "
           "complessità cognitiva elevata, troppi parametri booleani."),
    "I8": ("Codice duplicato o inutile che non contribuisce con nuova logica "
           "o comportamento: istruzioni senza effetto, mut non necessari, "
           "variabili/import non utilizzati, closure ridondanti."),
    "E1":  ("Pattern che possono portare a bug o comportamenti indefiniti "
            "a causa di operazioni aritmetiche: cast con perdita di segno o "
            "precisione, overflow, divisione intera non intenzionale."),
    "E2":  ("Uso improprio o mancante di attributi Rust che influenzano il "
            "comportamento o la stabilità del codice "
            "(deprecated, inline, allow senza giustificazione)."),
    "E3":  ("Codice che potrebbe non funzionare correttamente su piattaforme, "
            "versioni di Rust o ambienti diversi."),
    "E4":  ("Codice che gestisce gli errori nascondendo le cause radice o "
            "limitando la debuggabilità: wildcard su Err, Result ignorato, "
            "propagazione silenziosa."),
    "E5":  ("Codice con sintassi valida ma che riflette probabilmente un "
            "errore logico: condizioni always-true/false, operandi identici "
            "in confronti, bitmask inefficaci."),
    "E6":  ("Codice che rischia dangling pointer, buffer overflow, "
            "use-after-free o data race: operazioni unsafe fuori da blocchi "
            "espliciti, transmute non sicuri, puntatori grezzi."),
    "E7":  ("Codice che compila ed esegue correttamente ma produce esecuzione "
            "inefficiente: allocazioni ridondanti, clone non necessari, "
            "concatenazione di stringhe con +."),
    "E8":  ("Codice che può causare un panic a runtime a causa di operazioni "
            "non verificate: unwrap/expect su Option/Result, accessi a slice "
            "senza bounds-check."),
    "E9":  ("Codice che può causare comportamento indefinito o data race se "
            "usato su più thread: lock attraverso await, tipi non-Send "
            "condivisi, primitivi di sincronizzazione mal usati."),
    "E10": ("Codice che scarta le garanzie di tipo di Rust: cast impliciti "
            "tra tipi incompatibili, conversioni fn→numerico, "
            "convenzioni self errate."),
    "UNMAPPED": ("Lint non presenti nel dizionario di mappatura; richiedono "
                 "revisione manuale per essere assegnati alla categoria corretta."),
}

# ---------------------------------------------------------------------------
# DIZIONARIO DI MAPPATURA  lint_id → codice categoria
# ---------------------------------------------------------------------------

TAXONOMY_MAP: dict[str, str] = {

    # ── I1 · Convention violation ─────────────────────────────────────────
    "non_snake_case":                             "I1",
    "non_camel_case_types":                       "I1",
    "non_upper_case_globals":                     "I1",
    "clippy::module_name_repetitions":            "I1",
    "clippy::enum_variant_names":                 "I1",
    "clippy::struct_field_names":                 "I1",
    "clippy::non_ascii_literal":                  "I1",

    # ── I2 · Documentation issues ─────────────────────────────────────────
    "missing_docs":                               "I2",
    "clippy::missing_safety_doc":                 "I2",
    "clippy::missing_panics_doc":                 "I2",
    "clippy::missing_errors_doc":                 "I2",
    "clippy::undocumented_unsafe_blocks":         "I2",
    "clippy::missing_docs_in_private_items":      "I2",

    # ── I3 · Inflexible code ──────────────────────────────────────────────
    "clippy::type_complexity":                    "I3",
    "clippy::trait_duplication_in_bounds":        "I3",
    "clippy::type_repetition_in_bounds":          "I3",
    "clippy::large_types_passed_by_value":        "I3",
    "clippy::rc_buffer":                          "I3",

    # ── I4 · Misleading code ──────────────────────────────────────────────
    "clippy::suspicious_arithmetic_impl":         "I4",
    "clippy::suspicious_assignment_formatting":   "I4",
    "clippy::suspicious_else_formatting":         "I4",
    "clippy::suspicious_unary_op_formatting":     "I4",
    "clippy::clone_on_copy":                      "I4",
    "clippy::match_same_arms":                    "I4",
    "clippy::if_same_then_else":                  "I4",
    "clippy::suspicious_op_assign_impl":          "I4",

    # ── I5 · Non-idiomatic code ───────────────────────────────────────────
    "clippy::manual_map":                         "I5",
    "clippy::manual_flatten":                     "I5",
    "clippy::manual_ok_or":                       "I5",
    "clippy::manual_unwrap_or":                   "I5",
    "clippy::map_unwrap_or":                      "I5",
    "clippy::option_if_let_else":                 "I5",
    "clippy::match_like_matches_macro":           "I5",
    "clippy::use_self":                           "I5",
    "clippy::cloned_instead_of_copied":           "I5",
    "clippy::explicit_iter_loop":                 "I5",
    "clippy::explicit_into_iter_loop":            "I5",
    "clippy::while_let_on_iterator":              "I5",
    "clippy::while_let_loop":                     "I5",
    "clippy::question_mark":                      "I5",
    "clippy::manual_let_else":                    "I5",
    "clippy::needless_return":                    "I5",
    "clippy::toplevel_ref_arg":                   "I5",
    "clippy::collapsible_if":                     "I5",
    "clippy::collapsible_else_if":                "I5",
    "clippy::collapsible_match":                  "I5",
    "clippy::items_after_statements":             "I5",
    "clippy::print_with_newline":                 "I5",
    "clippy::println_empty_string":               "I5",
    "clippy::unnecessary_mut_passed":             "I5",   # passa &mut quando &T basta
    "clippy::zero_ptr":                           "I5",   # usa ptr::null()/null_mut() invece di 0 as *_
    "clippy::manual_swap":                        "I5",   # usa mem::swap() invece di una variabile temporanea

    # ── I6 · Non-production code ──────────────────────────────────────────
    "clippy::todo":                               "I6",
    "clippy::unimplemented":                      "I6",
    "clippy::panic":                              "I6",
    "clippy::dbg_macro":                          "I6",
    "clippy::print_stdout":                       "I6",
    "clippy::print_stderr":                       "I6",

    # ── I7 · Readability issues ───────────────────────────────────────────
    "clippy::cognitive_complexity":               "I7",
    "clippy::too_many_arguments":                 "I7",
    "clippy::too_many_lines":                     "I7",
    "clippy::fn_params_excessive_bools":          "I7",
    "clippy::struct_excessive_bools":             "I7",
    "clippy::match_wildcard_for_single_variants": "I7",
    "clippy::wildcard_enum_match_arm":            "I7",
    "clippy::single_match_else":                  "I7",
    "clippy::needless_bool":                      "I7",
    "clippy::large_enum_variant":                 "I7",

    # ── I8 · Redundant code ───────────────────────────────────────────────
    "clippy::no_effect":                          "I8",
    "clippy::unnecessary_operation":              "I8",
    "clippy::useless_let_if_seq":                 "I8",
    "clippy::double_neg":                         "I8",
    "clippy::identity_op":                        "I8",
    "clippy::needless_bool":                      "I8",   # also readability
    "clippy::redundant_pattern":                  "I8",
    "clippy::redundant_pattern_matching":         "I8",
    "clippy::redundant_closure":                  "I8",
    "clippy::redundant_closure_call":             "I8",
    "clippy::redundant_else":                     "I8",
    "clippy::redundant_field_names":              "I8",
    "clippy::redundant_static_lifetimes":         "I8",
    "clippy::needless_pass_by_ref_mut":           "I8",
    "clippy::unneeded_field_pattern":             "I8",
    "unused_mut":                                 "I8",
    "unused_variables":                           "I8",
    "dead_code":                                  "I8",
    "unused_imports":                             "I8",
    "unused_must_use":                            "I8",
    "clippy::unused_self":                        "I8",
    "clippy::unused_async":                       "I8",
    "clippy::unused_peekable":                    "I8",
    "path_statements":                            "I8",   # istruzione-espressione senza effetto (es. `x;`)
    "unused_assignments":                         "I8",   # assegnamento scritto ma mai letto

    # ── E1 · Arithmetic issues ────────────────────────────────────────────
    "clippy::cast_possible_truncation":           "E1",
    "clippy::cast_possible_wrap":                 "E1",
    "clippy::cast_sign_loss":                     "E1",
    "clippy::cast_lossless":                      "E1",
    "clippy::cast_precision_loss":                "E1",
    "clippy::integer_arithmetic":                 "E1",
    "clippy::integer_division":                   "E1",
    "clippy::arithmetic_side_effects":            "E1",
    "clippy::overflow_check_conditional":         "E1",
    "clippy::checked_conversions":                "E1",
    "clippy::manual_saturating_arithmetic":       "E1",
    "clippy::as_conversions":                     "E1",
    "clippy::modulo_one":                         "E1",
    "clippy::bad_bit_mask":                       "E1",
    "clippy::ineffective_bit_mask":               "E1",

    # ── E2 · Attribute issues ─────────────────────────────────────────────
    "deprecated":                                 "E2",
    "clippy::deprecated":                         "E2",
    "clippy::allow_attributes_without_reason":    "E2",
    "clippy::inline_always":                      "E2",
    "clippy::empty_line_after_outer_attr":        "E2",

    # ── E3 · Compatibility issues ─────────────────────────────────────────
    "clippy::unnecessary_cast":                   "E3",   # cross-platform cast
    "clippy::ptr_cast_constness":                 "E3",
    "clippy::as_ptr_cast_mut":                    "E3",

    # ── E4 · Error handling issues ────────────────────────────────────────
    "clippy::map_err_ignore":                     "E4",
    "clippy::let_underscore_must_use":            "E4",
    "clippy::match_wild_err_arm":                 "E4",
    "clippy::result_large_err":                   "E4",
    "clippy::try_err":                            "E4",

    # ── E5 · Logical issues ───────────────────────────────────────────────
    "clippy::logic_bug":                          "E5",
    "clippy::overly_complex_bool_expr":           "E5",
    "clippy::nonminimal_bool":                    "E5",
    "clippy::neg_cmp_op_on_partial_ord":          "E5",
    "clippy::float_cmp":                          "E5",
    "clippy::float_cmp_const":                    "E5",
    "clippy::absurd_extreme_comparisons":         "E5",
    "clippy::eq_op":                              "E5",
    "clippy::ifs_same_cond":                      "E5",

    # ── E6 · Memory safety ────────────────────────────────────────────────
    # E0133 = errore compilatore "use of unsafe code" (unsafe op fuori blocco)
    "E0133":                                      "E6",
    "clippy::unsafe_op_in_unsafe_fn":             "E6",
    "clippy::ptr_as_ptr":                         "E6",
    "clippy::ptr_offset_with_cast":               "E6",
    "clippy::transmute_ptr_to_ref":               "E6",
    "clippy::transmute_ptr_to_ptr":               "E6",
    "clippy::transmuting_null":                   "E6",
    "clippy::unsound_collection_transmute":       "E6",
    "clippy::not_unsafe_ptr_arg_deref":           "E6",
    # clippy::zero_ptr spostato in I5 (usa ptr::null/null_mut al posto di 0 as *_)
    "clippy::mut_from_ref":                       "E6",
    "clippy::mem_forget":                         "E6",
    "clippy::drop_ref":                           "E6",
    "clippy::forget_ref":                         "E6",
    "clippy::borrow_as_ptr":                      "E6",

    # ── E7 · Performance ──────────────────────────────────────────────────
    "clippy::redundant_allocation":               "E7",
    "clippy::unnecessary_to_owned":               "E7",
    "clippy::string_to_string":                   "E7",
    "clippy::str_to_string":                      "E7",
    "clippy::string_add":                         "E7",
    "clippy::string_add_assign":                  "E7",
    "clippy::string_lit_as_bytes":                "E7",
    "clippy::format_push_string":                 "E7",
    "clippy::useless_format":                     "E7",
    "clippy::needless_pass_by_value":             "E7",
    "clippy::large_stack_arrays":                 "E7",

    # ── E8 · Runtime Panic risks ──────────────────────────────────────────
    "clippy::unwrap_used":                        "E8",
    "clippy::expect_used":                        "E8",
    "clippy::indexing_slicing":                   "E8",
    "clippy::get_unwrap":                         "E8",
    "clippy::unwrap_in_result":                   "E8",
    "clippy::unreachable":                        "E8",
    "clippy::out_of_bounds_indexing":             "E8",

    # ── E9 · Thread safety ────────────────────────────────────────────────
    "clippy::await_holding_lock":                 "E9",
    "clippy::await_holding_refcell_ref":          "E9",
    "clippy::mutex_integer":                      "E9",
    "clippy::mutex_atomic":                       "E9",
    "clippy::arc_with_non_send_sync":             "E9",
    "clippy::non_send_fields_in_send_ty":         "E9",

    # ── E10 · Type safety ─────────────────────────────────────────────────
    "clippy::fn_to_numeric_cast":                 "E10",
    "clippy::fn_to_numeric_cast_with_truncation": "E10",
    "clippy::into_iter_on_ref":                   "E10",
    "clippy::wrong_self_convention":              "E10",
    "clippy::should_implement_trait":             "E10",
    "clippy::useless_conversion":                 "E10",
    "clippy::char_lit_as_u8":                     "E10",
    "clippy::vtable_address_comparisons":         "E10",
    "clippy::unsafe_removed_from_name":           "E10",
    "mutable_transmutes":                         "E10",  # trasmutazione che aggiunge mutabilità (&T → &mut T)
}

# ---------------------------------------------------------------------------
# CLASSIFICAZIONE
# ---------------------------------------------------------------------------

def classify_lint(lint_id: str) -> str:
    """
    Mappa un lint ID alla categoria della tassonomia.

    Strategia di lookup:
      1. Corrispondenza esatta case-sensitive (es. 'E0133')
      2. Corrispondenza esatta case-insensitive
      3. Corrispondenza per prefisso (es. lint con suffisso dinamico)
      4. Fallback UNMAPPED — stampa avviso su stderr per revisione futura
    """
    # 1. lookup esatto case-sensitive
    if lint_id in TAXONOMY_MAP:
        return TAXONOMY_MAP[lint_id]
    # 2. lookup case-insensitive
    key = lint_id.strip().lower()
    if key in TAXONOMY_MAP:
        return TAXONOMY_MAP[key]
    # 3. corrispondenza per prefisso
    for map_key, category in TAXONOMY_MAP.items():
        if key.startswith(map_key.lower()):
            return category
    # 4. fallback: avvisa l'utente così può aggiornare il dizionario
    print(
        f"  [UNMAPPED] lint non censito: '{lint_id}' → aggiungilo a TAXONOMY_MAP",
        file=sys.stderr,
    )
    return "UNMAPPED"

# ---------------------------------------------------------------------------
# PARSING
# ---------------------------------------------------------------------------

def parse_clippy_json(input_path: Path) -> list[dict]:
    """Legge il file NDJSON di cargo clippy e restituisce le violazioni."""
    violations: list[dict] = []

    with input_path.open(encoding="utf-8") as fh:
        for line_num, raw_line in enumerate(fh, start=1):
            raw_line = raw_line.strip()
            if not raw_line:
                continue
            try:
                obj = json.loads(raw_line)
            except json.JSONDecodeError as exc:
                print(f"  [WARN] Riga {line_num}: JSON non valido ({exc}), saltata.",
                      file=sys.stderr)
                continue

            if obj.get("reason") != "compiler-message":
                continue

            message_obj = obj.get("message", {})
            level = message_obj.get("level", "")
            if level not in ("warning", "error"):
                continue

            code_obj = message_obj.get("code") or {}
            lint_id: str = code_obj.get("code", "")
            if not lint_id:
                continue

            msg_text: str = message_obj.get("message", "")

            spans: list[dict] = message_obj.get("spans", [])
            if spans:
                primary_span = spans[0]
                file_name: str  = primary_span.get("file_name", "N/A")
                line_start: int = primary_span.get("line_start", 0)
            else:
                file_name  = "N/A"
                line_start = 0

            violations.append({
                "lint_id":       lint_id,
                "file_name":     file_name,
                "line_start":    line_start,
                "message":       msg_text,
                "category_code": classify_lint(lint_id),
            })

    return violations

# ---------------------------------------------------------------------------
# RIEPILOGO
# ---------------------------------------------------------------------------

def build_summary(violations: list[dict]) -> dict[str, int]:
    counts: dict[str, int] = defaultdict(int)
    for v in violations:
        counts[v["category_code"]] += 1
    return dict(counts)

# ---------------------------------------------------------------------------
# STAMPA TERMINALE
# ---------------------------------------------------------------------------

def print_summary(counts: dict[str, int], total: int) -> None:
    sep = "─" * 62
    print("\n" + sep)
    print("  RIEPILOGO TASSONOMIA — Tadesse et al.")
    print(sep)
    print(f"  {'Codice':<6}{'Categoria':<34}{'Warning':>8}")
    print(sep)
    print("  ── INTERNAL QUALITY " + "─" * 42)
    for code in INTERNAL_CODES:
        print(f"  {code:<6}{TAXONOMY[code]:<34}{counts.get(code, 0):>8}")
    print("  ── EXTERNAL QUALITY " + "─" * 42)
    for code in EXTERNAL_CODES:
        print(f"  {code:<6}{TAXONOMY[code]:<34}{counts.get(code, 0):>8}")
    print(sep)
    if counts.get("UNMAPPED", 0) > 0:
        print(f"  {'UNMAP':<6}{'Unmapped / Other':<34}{counts.get('UNMAPPED', 0):>8}")
        print(sep)
    print(f"  {'TOTALE':<40}{total:>8}")
    print(sep + "\n")

# ---------------------------------------------------------------------------
# CSS (stringa normale, NON f-string, per evitare conflitti con {})
# ---------------------------------------------------------------------------

_CSS_BASE = """
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap');

    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    body {
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
        background: #0d1117;
        color: #e2e8f0;
        min-height: 100vh;
        padding: 2rem;
        line-height: 1.5;
    }

    .container { max-width: 1440px; margin: 0 auto; }

    /* ── Header ── */
    .page-header {
        text-align: center;
        padding: 2.5rem 2rem;
        margin-bottom: 2rem;
        background: linear-gradient(135deg, #161b27 0%, #1a2035 50%, #161b27 100%);
        border-radius: 16px;
        border: 1px solid #2d3748;
    }
    .page-header h1 {
        font-size: 1.6rem;
        font-weight: 700;
        color: #f0f4ff;
        letter-spacing: -0.02em;
        margin-bottom: 0.35rem;
    }
    .page-header .subtitle {
        font-size: 0.875rem;
        color: #718096;
        margin-bottom: 1.75rem;
    }

    /* ── Stat cards ── */
    .stats-row {
        display: flex;
        justify-content: center;
        gap: 1rem;
        flex-wrap: wrap;
    }
    .stat-card {
        background: rgba(255,255,255,0.04);
        border: 1px solid rgba(255,255,255,0.08);
        border-radius: 12px;
        padding: 0.8rem 1.4rem;
        text-align: center;
        min-width: 120px;
    }
    .stat-card .val {
        font-size: 1.75rem;
        font-weight: 700;
        line-height: 1;
        margin-bottom: 0.25rem;
    }
    .stat-card .lbl {
        font-size: 0.68rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.06em;
        color: #718096;
    }
    .stat-total    .val { color: #63b3ed; }
    .stat-internal .val { color: #68d391; }
    .stat-external .val { color: #f687b3; }
    .stat-unmapped .val { color: #f6ad55; }

    /* ── Table wrapper ── */
    .table-wrap {
        background: #161b27;
        border-radius: 14px;
        border: 1px solid #2d3748;
        overflow: hidden;
        margin-bottom: 1.5rem;
    }

    /* ── Section divider bars ── */
    .section-bar {
        display: flex;
        align-items: center;
        gap: 0.6rem;
        padding: 0.6rem 1.25rem;
        font-size: 0.72rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.1em;
    }
    .section-bar.int { background: linear-gradient(90deg,#0d2a1e,#161b27); color: #68d391; border-top: 1px solid #2d3748; }
    .section-bar.ext { background: linear-gradient(90deg,#1e1040,#161b27); color: #b794f4; border-top: 2px solid #2d3748; }
    .section-bar.unm { background: linear-gradient(90deg,#2e1a06,#161b27); color: #f6ad55; border-top: 2px solid #2d3748; }

    /* ── Table ── */
    table { width: 100%; border-collapse: collapse; }

    thead th {
        padding: 0.72rem 1.1rem;
        font-size: 0.7rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.08em;
        color: #4a5568;
        background: rgba(0,0,0,0.25);
        border-bottom: 1px solid #2d3748;
        text-align: left;
    }
    thead th.center { text-align: center; }

    tbody td {
        padding: 0.82rem 1.1rem;
        border-bottom: 1px solid rgba(45,55,72,0.4);
        vertical-align: top;
        font-size: 0.82rem;
    }
    tbody tr:last-child td { border-bottom: none; }
    tbody tr:hover td { background: rgba(255,255,255,0.025); transition: background 0.15s; }

    .td-cat  { font-weight: 600; color: #cbd5e0; width: 210px; min-width: 180px; }
    .td-lint { width: 250px; min-width: 190px; }
    .td-cnt  { width: 88px; text-align: center; }
    .td-desc { color: #94a3b8; line-height: 1.65; }

    /* ── Lint tags ── */
    .lint-tag {
        display: inline-block;
        font-family: 'JetBrains Mono', 'Fira Code', monospace;
        font-size: 0.7rem;
        background: rgba(99,179,237,0.09);
        color: #63b3ed;
        border: 1px solid rgba(99,179,237,0.2);
        border-radius: 4px;
        padding: 0.12rem 0.42rem;
        margin: 1px 0;
        white-space: nowrap;
        line-height: 1.7;
    }
    .lint-tag.compiler {
        background: rgba(246,173,85,0.09);
        color: #f6ad55;
        border-color: rgba(246,173,85,0.25);
    }
    .no-lint { color: #4a5568; font-style: italic; }

    /* ── Count badge ── */
    .badge {
        display: inline-block;
        font-weight: 700;
        font-size: 0.82rem;
        border-radius: 20px;
        padding: 0.18rem 0.72rem;
        min-width: 2.2rem;
        text-align: center;
    }
    .badge-zero { background: rgba(74,85,104,0.2);   color: #4a5568; border: 1px solid rgba(74,85,104,0.3); }
    .badge-low  { background: rgba(246,173,85,0.13);  color: #f6ad55; border: 1px solid rgba(246,173,85,0.28); }
    .badge-high { background: rgba(252,129,129,0.13); color: #fc8181; border: 1px solid rgba(252,129,129,0.28); }
    .badge-unm  { background: rgba(246,173,85,0.18);  color: #ed8936; border: 1px solid rgba(246,173,85,0.4); }

    /* ── Row states ── */
    .row-zero td { opacity: 0.5; }
    .row-unm  td { background: rgba(246,173,85,0.03); }
    .row-total td {
        background: rgba(0,0,0,0.3);
        border-top: 2px solid #2d3748 !important;
        font-weight: 700;
        font-size: 0.9rem;
    }
    .row-total .td-cnt { color: #63b3ed; font-size: 1.05rem; text-align: center; }

    /* ── Footer ── */
    .page-footer { text-align: center; margin-top: 2rem; font-size: 0.72rem; color: #4a5568; }
"""

_CSS_REPORT = _CSS_BASE + """
    .file-card {
        background: #161b27;
        border-radius: 12px;
        border: 1px solid #2d3748;
        overflow: hidden;
        margin-bottom: 1.25rem;
    }
    .file-hdr {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.72rem 1.2rem;
        background: rgba(0,0,0,0.3);
        border-bottom: 1px solid #2d3748;
    }
    .file-path {
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.78rem;
        color: #90cdf4;
        flex: 1;
        word-break: break-all;
    }
    .file-cnt {
        font-size: 0.7rem;
        font-weight: 600;
        background: rgba(99,179,237,0.1);
        color: #63b3ed;
        border: 1px solid rgba(99,179,237,0.2);
        border-radius: 10px;
        padding: 0.12rem 0.55rem;
        white-space: nowrap;
    }
    .td-line {
        font-family: 'JetBrains Mono', monospace;
        color: #718096;
        white-space: nowrap;
        width: 70px;
        font-size: 0.78rem;
    }
    .td-catbadge { width: 240px; }
    .td-lintcode { width: 220px; }
    .td-msg { color: #94a3b8; line-height: 1.55; font-size: 0.8rem; }
    .cat-pill {
        display: inline-block;
        border-radius: 6px;
        padding: 0.18rem 0.55rem;
        font-size: 0.7rem;
        font-weight: 700;
        white-space: nowrap;
    }
    .lint-code {
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.72rem;
        background: rgba(99,179,237,0.08);
        color: #63b3ed;
        border: 1px solid rgba(99,179,237,0.15);
        border-radius: 4px;
        padding: 0.12rem 0.38rem;
        white-space: nowrap;
    }
"""

# Palette colori per categoria
_CAT_PALETTE: dict[str, tuple[tuple[int, int, int], str]] = {
    "I1":  ((233,216,253), "#b794f4"),
    "I2":  ((198,246,213), "#68d391"),
    "I3":  ((190,227,248), "#63b3ed"),
    "I4":  ((254,252,191), "#d69e2e"),
    "I5":  ((178,245,234), "#4fd1c5"),
    "I6":  ((254,215,215), "#fc8181"),
    "I7":  ((214,188,250), "#9f7aea"),
    "I8":  ((226,232,240), "#a0aec0"),
    "E1":  ((250,240,137), "#ecc94b"),
    "E2":  ((154,230,180), "#48bb78"),
    "E3":  ((129,230,217), "#38b2ac"),
    "E4":  ((246,173, 85), "#ed8936"),
    "E5":  ((251,211,141), "#d69e2e"),
    "E6":  ((252,129,129), "#fc8181"),
    "E7":  ((104,211,145), "#38a169"),
    "E8":  ((246,135, 85), "#e53e3e"),
    "E9":  (( 99,179,237), "#3182ce"),
    "E10": ((160,174,192), "#718096"),
    "UNMAPPED": ((246,173,85), "#ed8936"),
}

def _cat_pill(code: str) -> str:
    rgb, fg = _CAT_PALETTE.get(code, ((160,174,192), "#718096"))
    r, g, b = rgb
    name = TAXONOMY.get(code, "Unmapped")
    return (
        f'<span class="cat-pill" style="'
        f'background:rgba({r},{g},{b},0.13);'
        f'color:{fg};'
        f'border:1px solid rgba({r},{g},{b},0.3);">'
        f'{html_lib.escape(code)} &ndash; {html_lib.escape(name)}</span>'
    )

def _lint_tags(lint_list: list[str]) -> str:
    if not lint_list:
        return '<span class="no-lint">&mdash;</span>'
    parts = []
    for lint in lint_list:
        # I lint del compilatore (es. E0133, non_snake_case) hanno stile diverso
        is_compiler = not lint.startswith("clippy")
        cls = "lint-tag compiler" if is_compiler else "lint-tag"
        parts.append(f'<code class="{cls}">{html_lib.escape(lint)}</code>')
    return "<br>".join(parts)

def _badge(count: int, style: str = "") -> str:
    if style:
        cls = f"badge badge-{style}"
    elif count == 0:
        cls = "badge badge-zero"
    elif count >= 10:
        cls = "badge badge-high"
    else:
        cls = "badge badge-low"
    return f'<span class="{cls}">{count}</span>'

# ---------------------------------------------------------------------------
# HTML — TABELLA RIEPILOGATIVA
# ---------------------------------------------------------------------------

def write_html_taxonomy_table(
    violations: list[dict],
    counts: dict[str, int],
    output_path: Path,
) -> None:

    lints_per_cat: dict[str, set[str]] = defaultdict(set)
    for v in violations:
        lints_per_cat[v["category_code"]].add(v["lint_id"])

    total    = sum(counts.values())
    int_tot  = sum(counts.get(c, 0) for c in INTERNAL_CODES)
    ext_tot  = sum(counts.get(c, 0) for c in EXTERNAL_CODES)
    unm_tot  = counts.get("UNMAPPED", 0)
    ts       = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    def make_rows(codes: list[str]) -> str:
        out = ""
        for code in codes:
            lint_list = sorted(lints_per_cat.get(code, set()))
            count     = counts.get(code, 0)
            desc      = html_lib.escape(TAXONOMY_DESCRIPTIONS.get(code, ""))
            cat_label = html_lib.escape(f"{code} \u2013 {TAXONOMY.get(code,'')}")
            row_cls   = "row-zero" if count == 0 else ""
            out += (
                f'<tr class="{row_cls}">'
                f'<td class="td-cat">{cat_label}</td>'
                f'<td class="td-lint">{_lint_tags(lint_list)}</td>'
                f'<td class="td-cnt">{_badge(count)}</td>'
                f'<td class="td-desc">{desc}</td>'
                f'</tr>\n'
            )
        return out

    rows_int = make_rows(INTERNAL_CODES)
    rows_ext = make_rows(EXTERNAL_CODES)

    unmapped_html = ""
    if unm_tot > 0:
        lint_list = sorted(lints_per_cat.get("UNMAPPED", set()))
        desc      = html_lib.escape(TAXONOMY_DESCRIPTIONS.get("UNMAPPED", ""))
        unmapped_html = (
            '<tr><td colspan="4" style="padding:0">'
            '<div class="section-bar unm">&#9888; Non Mappati / Unmapped</div></td></tr>\n'
            f'<tr class="row-unm">'
            f'<td class="td-cat">UNMAPPED &ndash; Unmapped / Other</td>'
            f'<td class="td-lint">{_lint_tags(lint_list)}</td>'
            f'<td class="td-cnt">{_badge(unm_tot, "unm")}</td>'
            f'<td class="td-desc">{desc}</td>'
            f'</tr>\n'
        )

    doc = f"""<!DOCTYPE html>
<html lang="it">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Tabella Riepilogativa &mdash; Tassonomia Qualit&agrave; Tadesse et al.</title>
  <style>{_CSS_BASE}</style>
</head>
<body>
<div class="container">

  <div class="page-header">
    <h1>Tabella Riepilogativa &mdash; Tassonomia Qualit&agrave;</h1>
    <p class="subtitle">Tadesse et al. &middot; 18 categorie &middot; Rust Code Quality Analysis</p>
    <div class="stats-row">
      <div class="stat-card stat-total">
        <div class="val">{total}</div><div class="lbl">Totale</div>
      </div>
      <div class="stat-card stat-internal">
        <div class="val">{int_tot}</div><div class="lbl">Internal Quality</div>
      </div>
      <div class="stat-card stat-external">
        <div class="val">{ext_tot}</div><div class="lbl">External Quality</div>
      </div>
      <div class="stat-card stat-unmapped">
        <div class="val">{unm_tot}</div><div class="lbl">Non Mappati</div>
      </div>
    </div>
  </div>

  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Categoria Tassonomica</th>
          <th>Lint Rilevato</th>
          <th class="center">Conteggio</th>
          <th>Descrizione</th>
        </tr>
      </thead>
      <tbody>
        <tr><td colspan="4" style="padding:0">
          <div class="section-bar int">&#9632; Internal Quality</div>
        </td></tr>
        {rows_int}
        <tr><td colspan="4" style="padding:0">
          <div class="section-bar ext">&#9632; External Quality</div>
        </td></tr>
        {rows_ext}
        {unmapped_html}
        <tr class="row-total">
          <td colspan="2" class="td-cat">TOTALE</td>
          <td class="td-cnt">{total}</td>
          <td></td>
        </tr>
      </tbody>
    </table>
  </div>

  <div class="page-footer">
    Generato il {ts} &middot; clippy_taxonomy_parser.py &middot; Tadesse et al. Taxonomy
  </div>

</div>
</body>
</html>"""

    with output_path.open("w", encoding="utf-8") as fh:
        fh.write(doc)


# ---------------------------------------------------------------------------
# HTML — REPORT DETTAGLIATO
# ---------------------------------------------------------------------------

def write_html_report(violations: list[dict], output_path: Path) -> None:
    sorted_v = sorted(violations, key=lambda v: (v["file_name"], v["line_start"]))
    total    = len(sorted_v)
    ts       = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    files: dict[str, list[dict]] = defaultdict(list)
    for v in sorted_v:
        files[v["file_name"]].append(v)

    file_sections = ""
    for fname, viols in files.items():
        rows = ""
        for v in viols:
            code = v["category_code"]
            msg  = html_lib.escape(v["message"])
            lint = html_lib.escape(v["lint_id"])
            rows += (
                f'<tr>'
                f'<td class="td-line">:{v["line_start"]}</td>'
                f'<td class="td-catbadge">{_cat_pill(code)}</td>'
                f'<td class="td-lintcode"><code class="lint-code">{lint}</code></td>'
                f'<td class="td-msg">{msg}</td>'
                f'</tr>\n'
            )
        n = len(viols)
        file_sections += f"""
  <div class="file-card">
    <div class="file-hdr">
      <span>&#128196;</span>
      <span class="file-path">{html_lib.escape(fname)}</span>
      <span class="file-cnt">{n} violazioni</span>
    </div>
    <table>
      <thead>
        <tr><th>Riga</th><th>Categoria</th><th>Lint ID</th><th>Messaggio</th></tr>
      </thead>
      <tbody>{rows}</tbody>
    </table>
  </div>"""

    doc = f"""<!DOCTYPE html>
<html lang="it">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Report Dettagliato &mdash; Clippy Taxonomy Parser</title>
  <style>{_CSS_REPORT}</style>
</head>
<body>
<div class="container">

  <div class="page-header">
    <h1>Report Dettagliato &mdash; Clippy Taxonomy Parser</h1>
    <p class="subtitle">Tassonomia: Tadesse et al. (18 categorie) &middot; Rust Code Analysis</p>
    <div class="stats-row">
      <div class="stat-card stat-total">
        <div class="val">{total}</div><div class="lbl">Violazioni Totali</div>
      </div>
      <div class="stat-card stat-internal">
        <div class="val">{len(files)}</div><div class="lbl">File Analizzati</div>
      </div>
    </div>
  </div>

  {file_sections}

  <div class="page-footer">
    Generato il {ts} &middot; clippy_taxonomy_parser.py
  </div>

</div>
</body>
</html>"""

    with output_path.open("w", encoding="utf-8") as fh:
        fh.write(doc)

# ---------------------------------------------------------------------------
# DATI DEMO
# ---------------------------------------------------------------------------

def _make_demo_violations() -> list[dict]:
    """Violazioni fittizie per test senza clippy_results.json."""
    demo: list[dict] = []
    def add(lint, cat, n, file="src/lib.rs", start=1):
        for i in range(n):
            demo.append({"lint_id": lint, "file_name": file,
                         "line_start": start+i, "message": f"(demo) {lint}",
                         "category_code": cat})
    # Internal Quality
    add("non_snake_case",              "I1", 3, "src/lib.rs",    30)
    add("clippy::missing_safety_doc",  "I2", 2, "src/lib.rs",    75)
    add("clippy::type_complexity",     "I3", 1, "src/api.rs",    20)
    add("clippy::clone_on_copy",       "I4", 2, "src/math.rs",   15)
    add("clippy::needless_return",     "I5", 3, "src/lib.rs",    55)
    add("clippy::toplevel_ref_arg",    "I5", 2, "src/ffi.rs",    18)
    add("clippy::todo",                "I6", 2, "src/main.rs",   10)
    add("clippy::cognitive_complexity","I7", 4, "src/lib.rs",   110)
    add("unused_variables",            "I8", 2, "src/lib.rs",   200)
    add("unused_mut",                  "I8", 5, "src/gen.rs",    40)
    add("clippy::no_effect",           "I8", 4, "src/gen.rs",    60)
    # External Quality
    add("clippy::cast_sign_loss",      "E1", 5, "src/conv.rs",   20)
    add("clippy::map_err_ignore",      "E4", 4, "src/io.rs",      9)
    add("clippy::float_cmp",           "E5", 2, "src/math.rs",   15)
    add("E0133",                       "E6",40, "src/lib.rs",    10)
    add("clippy::string_add",          "E7", 3, "src/fmt.rs",    12)
    add("clippy::unwrap_used",         "E8",12, "src/main.rs",   20)
    add("clippy::await_holding_lock",  "E9", 1, "src/async.rs",   5)
    add("clippy::wrong_self_convention","E10",3,"src/api.rs",    11)
    return demo

# ---------------------------------------------------------------------------
# ENTRY POINT
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(
        description="Mappa i warning di cargo clippy sulle 18 categorie Tadesse et al."
    )
    parser.add_argument("--demo", action="store_true",
        help="Usa dati fittizi (non richiede clippy_results.json)")
    parser.add_argument("--input", "-i", default=INPUT_FILE,
        help=f"Percorso del file JSON di input (default: {INPUT_FILE})")
    args = parser.parse_args()

    output_path  = Path(OUTPUT_FILE)
    summary_path = Path(SUMMARY_FILE)

    if args.demo:
        print("[INFO] Modalità demo — dati fittizi in uso.")
        violations = _make_demo_violations()
    else:
        input_path = Path(args.input)
        if not input_path.exists():
            print(
                f"[ERRORE] File '{input_path}' non trovato.\n"
                f"\n"
                f"  Genera il file con:\n"
                f"    cargo clippy --message-format=json 2>&1 | tee {INPUT_FILE}\n"
                f"\n"
                f"  Oppure testa con dati fittizi:\n"
                f"    python3 {Path(__file__).name} --demo",
                file=sys.stderr,
            )
            return 1
        print(f"[INFO] Parsing di '{input_path}' in corso…")
        violations = parse_clippy_json(input_path)
        if not violations:
            print("[WARN] Nessuna violazione rilevata — tabella generata con tutti zeri.")

    total  = len(violations)
    counts = build_summary(violations)

    print(f"[INFO] Violazioni rilevate: {total}")
    print_summary(counts, total)

    write_html_report(violations, output_path)
    print(f"[INFO] Report dettagliato    → '{output_path}'")

    write_html_taxonomy_table(violations, counts, summary_path)
    print(f"[INFO] Tabella riepilogativa → '{summary_path}'")

    return 0


if __name__ == "__main__":
    sys.exit(main())