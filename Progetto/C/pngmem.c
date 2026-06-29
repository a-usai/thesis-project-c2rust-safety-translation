#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <limits.h>

/* --- Mock delle dipendenze di libpng per C2Rust --- */
typedef size_t png_alloc_size_t;
#define PNG_SIZE_MAX SIZE_MAX

typedef struct png_struct_def png_struct;

struct png_struct_def {
    void *mem_ptr;
};

/* Stub per la gestione errori */
void png_error(const png_struct *png_ptr, const char *error_message) {
    exit(1);
}

void png_warning(const png_struct *png_ptr, const char *warning_message) {
    /* Nessuna azione */
}

/* ---------------------------------------------------------------------- */
/* Logica originale di pngmem.c, pulita per la traduzione standalone      */
/* ---------------------------------------------------------------------- */

void png_destroy_png_struct(png_struct *png_ptr)
{
   if (png_ptr != NULL)
   {
      memset(png_ptr, 0, sizeof(png_struct));
      free(png_ptr);
   }
}

void *png_malloc_base(const png_struct *png_ptr, png_alloc_size_t size)
{
   if (size > PNG_SIZE_MAX) return NULL;
   return malloc((size_t)size);
}

void *png_calloc(const png_struct *png_ptr, png_alloc_size_t size)
{
   void *ret = png_malloc_base(png_ptr, size);
   if (ret != NULL)
      memset(ret, 0, size);
   return ret;
}

static void *png_malloc_array_checked(const png_struct *png_ptr, int nelements, size_t element_size)
{
   png_alloc_size_t req = (png_alloc_size_t)nelements;
   if (req <= PNG_SIZE_MAX/element_size)
      return png_malloc_base(png_ptr, req * element_size);
   return NULL;
}

void *png_malloc_array(const png_struct *png_ptr, int nelements, size_t element_size)
{
   if (nelements <= 0 || element_size == 0)
      png_error(png_ptr, "internal error: array alloc");
   return png_malloc_array_checked(png_ptr, nelements, element_size);
}

void *png_realloc_array(const png_struct *png_ptr, const void *old_array, int old_elements, int add_elements, size_t element_size)
{
   if (add_elements <= 0 || element_size == 0 || old_elements < 0 || (old_array == NULL && old_elements > 0))
      png_error(png_ptr, "internal error: array realloc");

   if (add_elements <= INT_MAX - old_elements)
   {
      void *new_array = png_malloc_array_checked(png_ptr, old_elements+add_elements, element_size);
      if (new_array != NULL)
      {
         if (old_elements > 0)
            memcpy(new_array, old_array, element_size*(unsigned)old_elements);
         memset((char*)new_array + element_size*(unsigned)old_elements, 0, element_size*(unsigned)add_elements);
         return new_array;
      }
   }
   return NULL;
}

void *png_malloc(const png_struct *png_ptr, png_alloc_size_t size)
{
   void *ret;
   if (png_ptr == NULL) return NULL;
   ret = png_malloc_base(png_ptr, size);
   if (ret == NULL) png_error(png_ptr, "Out of memory");
   return ret;
}

void *png_malloc_warn(const png_struct *png_ptr, png_alloc_size_t size)
{
   if (png_ptr != NULL)
   {
      void *ret = png_malloc_base(png_ptr, size);
      if (ret != NULL) return ret;
      png_warning(png_ptr, "Out of memory");
   }
   return NULL;
}

void png_free(const png_struct *png_ptr, void *ptr)
{
   if (png_ptr == NULL || ptr == NULL) return;
   free(ptr);
}