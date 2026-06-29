#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_DICT_SIZE 4096
#define MAX_STR_LEN 1000

typedef struct {
char *sequence;
int code;
} DictEntry;

DictEntry dictionary[MAX_DICT_SIZE];
int dict_size = 256;

void initializeTable() {
for (int i = 0; i < dict_size; i++) {
dictionary[i].sequence = (char *)malloc(2);
sprintf(dictionary[i].sequence, "%c", i);
dictionary[i].code = i;
}
}

int findCode(char *str) {
for (int i = 0; i < dict_size; i++) {
if (strcmp(dictionary[i].sequence, str) == 0) {
return dictionary[i].code;
}
}
return -1;
}

void addEntry(char *str) {
if (dict_size < MAX_DICT_SIZE) {
dictionary[dict_size].sequence = (char *)malloc(strlen(str) + 1);
strcpy(dictionary[dict_size].sequence, str);
dictionary[dict_size].code = dict_size;
dict_size++;
}
}

void compress(char *input) {
char current[MAX_STR_LEN] = "";
char next;
int code;

for (int i = 0; i < strlen(input); i++) {
next = input[i];
char temp[MAX_STR_LEN];
strcpy(temp, current);
strncat(temp, &next, 1);

if (findCode(temp) != -1) {
strcpy(current, temp);
} else {
code = findCode(current);
printf("%d ", code);
addEntry(temp);
strcpy(current, &next);
}
}
code = findCode(current);
printf("%d\n", code);
}

void freeTable() {
for (int i = 0; i < dict_size; i++) {
free(dictionary[i].sequence);
}
}

int main() {
char input[] = "TOBEORNOTTOBEORTOBEORNOT";
initializeTable();
compress(input);
freeTable();
return 0;
}