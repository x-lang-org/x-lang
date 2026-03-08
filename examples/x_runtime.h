/*
 * X-Lang C23 Runtime
 * Tagged-value runtime for dynamically typed X language programs.
 */
#ifndef X_RUNTIME_H
#define X_RUNTIME_H

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>
#include <math.h>
#include <ctype.h>
#ifdef _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <process.h>
#include <tlhelp32.h>
#include <psapi.h>
#else
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <sys/utsname.h>
#include <sys/sysinfo.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <pwd.h>
#include <grp.h>
#include <signal.h>
#include <time.h>
#include <dirent.h>
#endif

/* ========== Tagged Value ========== */

typedef enum {
    X_INT = 0,
    X_FLOAT,
    X_BOOL,
    X_STRING,
    X_ARRAY,
    X_MAP,
    X_NULL,
    X_NONE,
} XValueTag;

typedef struct XArray XArray;
typedef struct XMap XMap;
typedef struct XMapEntry XMapEntry;

typedef struct XValue {
    XValueTag tag;
    union {
        int64_t i;
        double f;
        bool b;
        char *s;
        XArray *arr;
        XMap *map;
    } as;
} XValue;

struct XArray {
    XValue *items;
    int64_t length;
    int64_t capacity;
    int64_t refcount;
};

struct XMapEntry {
    char *key;
    XValue value;
    struct XMapEntry *next;
};

struct XMap {
    XMapEntry **buckets;
    int64_t num_buckets;
    int64_t size;
    int64_t refcount;
};

/* ========== Value constructors ========== */

static inline XValue x_int(int64_t v) {
    XValue r; r.tag = X_INT; r.as.i = v; return r;
}

static inline XValue x_float(double v) {
    XValue r; r.tag = X_FLOAT; r.as.f = v; return r;
}

static inline XValue x_bool(bool v) {
    XValue r; r.tag = X_BOOL; r.as.b = v; return r;
}

static inline XValue x_null(void) {
    XValue r; r.tag = X_NULL; r.as.i = 0; return r;
}

static inline XValue x_none(void) {
    XValue r; r.tag = X_NONE; r.as.i = 0; return r;
}

static inline XValue x_string(const char *s) {
    XValue r;
    r.tag = X_STRING;
    r.as.s = (char *)malloc(strlen(s) + 1);
    strcpy(r.as.s, s);
    return r;
}

static inline XValue x_string_own(char *s) {
    XValue r; r.tag = X_STRING; r.as.s = s; return r;
}

/* ========== Array ========== */

static inline XArray *x_array_new(int64_t cap) {
    XArray *a = (XArray *)malloc(sizeof(XArray));
    a->length = 0;
    a->capacity = cap > 4 ? cap : 4;
    a->items = (XValue *)malloc(sizeof(XValue) * a->capacity);
    a->refcount = 1;
    return a;
}

static inline XValue x_array_val(XArray *a) {
    XValue r; r.tag = X_ARRAY; r.as.arr = a; return r;
}

static inline void x_array_push(XArray *a, XValue v) {
    if (a->length >= a->capacity) {
        a->capacity *= 2;
        a->items = (XValue *)realloc(a->items, sizeof(XValue) * a->capacity);
    }
    a->items[a->length++] = v;
}

static inline XValue x_array_get(XArray *a, int64_t idx) {
    if (idx < 0 || idx >= a->length) {
        fprintf(stderr, "Array index out of bounds: %lld (length %lld)\n",
                (long long)idx, (long long)a->length);
        exit(1);
    }
    return a->items[idx];
}

static inline void x_array_set(XArray *a, int64_t idx, XValue v) {
    if (idx < 0 || idx >= a->length) {
        fprintf(stderr, "Array index out of bounds: %lld (length %lld)\n",
                (long long)idx, (long long)a->length);
        exit(1);
    }
    a->items[idx] = v;
}

/* ========== Map ========== */

static inline uint64_t x_hash_str(const char *s) {
    uint64_t h = 5381;
    while (*s) { h = h * 33 + (unsigned char)*s; s++; }
    return h;
}

static inline XMap *x_map_new(void) {
    XMap *m = (XMap *)malloc(sizeof(XMap));
    m->num_buckets = 64;
    m->buckets = (XMapEntry **)calloc(m->num_buckets, sizeof(XMapEntry *));
    m->size = 0;
    m->refcount = 1;
    return m;
}

static inline XValue x_map_val(XMap *m) {
    XValue r; r.tag = X_MAP; r.as.map = m; return r;
}

static inline void x_map_set(XMap *m, const char *key, XValue val) {
    uint64_t h = x_hash_str(key) % m->num_buckets;
    XMapEntry *e = m->buckets[h];
    while (e) {
        if (strcmp(e->key, key) == 0) {
            e->value = val;
            return;
        }
        e = e->next;
    }
    XMapEntry *ne = (XMapEntry *)malloc(sizeof(XMapEntry));
    ne->key = (char *)malloc(strlen(key) + 1);
    strcpy(ne->key, key);
    ne->value = val;
    ne->next = m->buckets[h];
    m->buckets[h] = ne;
    m->size++;
}

static inline XValue x_map_get(XMap *m, const char *key) {
    uint64_t h = x_hash_str(key) % m->num_buckets;
    XMapEntry *e = m->buckets[h];
    while (e) {
        if (strcmp(e->key, key) == 0) return e->value;
        e = e->next;
    }
    return x_int(0);
}

static inline bool x_map_contains(XMap *m, const char *key) {
    uint64_t h = x_hash_str(key) % m->num_buckets;
    XMapEntry *e = m->buckets[h];
    while (e) {
        if (strcmp(e->key, key) == 0) return true;
        e = e->next;
    }
    return false;
}

static inline XValue x_map_keys(XMap *m) {
    XArray *keys = x_array_new(m->size);
    for (int64_t i = 0; i < m->num_buckets; i++) {
        XMapEntry *e = m->buckets[i];
        while (e) {
            x_array_push(keys, x_string(e->key));
            e = e->next;
        }
    }
    return x_array_val(keys);
}

/* ========== Type coercions ========== */

static inline double x_as_float(XValue v) {
    if (v.tag == X_FLOAT) return v.as.f;
    if (v.tag == X_INT) return (double)v.as.i;
    if (v.tag == X_BOOL) return v.as.b ? 1.0 : 0.0;
    return 0.0;
}

static inline int64_t x_as_int(XValue v) {
    if (v.tag == X_INT) return v.as.i;
    if (v.tag == X_FLOAT) return (int64_t)v.as.f;
    if (v.tag == X_BOOL) return v.as.b ? 1 : 0;
    return 0;
}

static inline bool x_as_bool(XValue v) {
    switch (v.tag) {
        case X_BOOL: return v.as.b;
        case X_INT: return v.as.i != 0;
        case X_FLOAT: return v.as.f != 0.0;
        case X_STRING: return v.as.s != NULL && v.as.s[0] != '\0';
        case X_NULL: case X_NONE: return false;
        default: return true;
    }
}

static inline const char *x_as_str(XValue v) {
    if (v.tag == X_STRING) return v.as.s;
    return "";
}

/* ========== Print ========== */

static inline void x_print(XValue v) {
    switch (v.tag) {
        case X_INT: printf("%lld\n", (long long)v.as.i); break;
        case X_FLOAT: printf("%g\n", v.as.f); break;
        case X_BOOL: printf("%s\n", v.as.b ? "true" : "false"); break;
        case X_STRING: printf("%s\n", v.as.s); break;
        case X_NULL: printf("null\n"); break;
        case X_NONE: printf("none\n"); break;
        case X_ARRAY: printf("[array len=%lld]\n", (long long)v.as.arr->length); break;
        case X_MAP: printf("[map size=%lld]\n", (long long)v.as.map->size); break;
    }
}

static inline void x_print_inline(XValue v) {
    switch (v.tag) {
        case X_INT: printf("%lld", (long long)v.as.i); break;
        case X_FLOAT: printf("%g", v.as.f); break;
        case X_BOOL: printf("%s", v.as.b ? "true" : "false"); break;
        case X_STRING: printf("%s", v.as.s); break;
        default: break;
    }
}

/* ========== String operations ========== */

static inline XValue x_concat(XValue a, XValue b) {
    const char *sa = x_as_str(a);
    const char *sb = x_as_str(b);
    size_t la = strlen(sa), lb = strlen(sb);
    char *buf = (char *)malloc(la + lb + 1);
    memcpy(buf, sa, la);
    memcpy(buf + la, sb, lb);
    buf[la + lb] = '\0';
    return x_string_own(buf);
}

static inline XValue x_to_string(XValue v) {
    char buf[64];
    switch (v.tag) {
        case X_INT: snprintf(buf, sizeof(buf), "%lld", (long long)v.as.i); return x_string(buf);
        case X_FLOAT: snprintf(buf, sizeof(buf), "%g", v.as.f); return x_string(buf);
        case X_BOOL: return x_string(v.as.b ? "true" : "false");
        case X_STRING: return v;
        case X_NULL: return x_string("null");
        case X_NONE: return x_string("none");
        default: return x_string("[object]");
    }
}

static inline XValue x_char_at(XValue s, XValue idx) {
    const char *str = x_as_str(s);
    int64_t i = x_as_int(idx);
    int64_t slen = (int64_t)strlen(str);
    if (i < 0 || i >= slen) return x_string("");
    char buf[2] = { str[i], '\0' };
    return x_string(buf);
}

static inline XValue x_substring(XValue s, XValue start, XValue end) {
    const char *str = x_as_str(s);
    int64_t slen = (int64_t)strlen(str);
    int64_t st = x_as_int(start);
    int64_t en = x_as_int(end);
    if (st < 0) st = 0;
    if (en > slen) en = slen;
    if (st >= en) return x_string("");
    int64_t rlen = en - st;
    char *buf = (char *)malloc(rlen + 1);
    memcpy(buf, str + st, rlen);
    buf[rlen] = '\0';
    return x_string_own(buf);
}

static inline XValue x_len(XValue v) {
    if (v.tag == X_STRING) return x_int((int64_t)strlen(v.as.s));
    if (v.tag == X_ARRAY) return x_int(v.as.arr->length);
    if (v.tag == X_MAP) return x_int(v.as.map->size);
    return x_int(0);
}

static inline XValue x_str_upper(XValue v) {
    const char *s = x_as_str(v);
    size_t n = strlen(s);
    char *buf = (char *)malloc(n + 1);
    for (size_t i = 0; i < n; i++) buf[i] = toupper((unsigned char)s[i]);
    buf[n] = '\0';
    return x_string_own(buf);
}

static inline XValue x_str_lower(XValue v) {
    const char *s = x_as_str(v);
    size_t n = strlen(s);
    char *buf = (char *)malloc(n + 1);
    for (size_t i = 0; i < n; i++) buf[i] = tolower((unsigned char)s[i]);
    buf[n] = '\0';
    return x_string_own(buf);
}

static inline XValue x_str_trim(XValue v) {
    const char *s = x_as_str(v);
    while (*s && isspace((unsigned char)*s)) s++;
    const char *end = s + strlen(s) - 1;
    while (end > s && isspace((unsigned char)*end)) end--;
    size_t n = end - s + 1;
    char *buf = (char *)malloc(n + 1);
    memcpy(buf, s, n);
    buf[n] = '\0';
    return x_string_own(buf);
}

static inline XValue x_str_split(XValue str_v, XValue delim_v) {
    const char *s = x_as_str(str_v);
    const char *d = x_as_str(delim_v);
    size_t dlen = strlen(d);
    XArray *arr = x_array_new(8);
    if (dlen == 0) {
        x_array_push(arr, x_string(s));
        return x_array_val(arr);
    }
    const char *pos = s;
    const char *found;
    while ((found = strstr(pos, d)) != NULL) {
        size_t seg = found - pos;
        char *buf = (char *)malloc(seg + 1);
        memcpy(buf, pos, seg);
        buf[seg] = '\0';
        x_array_push(arr, x_string_own(buf));
        pos = found + dlen;
    }
    x_array_push(arr, x_string(pos));
    return x_array_val(arr);
}

static inline XValue x_str_starts_with(XValue str_v, XValue prefix_v) {
    const char *s = x_as_str(str_v);
    const char *p = x_as_str(prefix_v);
    return x_bool(strncmp(s, p, strlen(p)) == 0);
}

static inline XValue x_str_contains(XValue str_v, XValue sub_v) {
    return x_bool(strstr(x_as_str(str_v), x_as_str(sub_v)) != NULL);
}

static inline XValue x_str_find(XValue str_v, XValue sub_v) {
    const char *s = x_as_str(str_v);
    const char *sub = x_as_str(sub_v);
    const char *p = strstr(s, sub);
    if (p) return x_int((int64_t)(p - s));
    return x_int(-1);
}

static inline XValue x_str_replace(XValue str_v, XValue old_v, XValue new_v) {
    const char *s = x_as_str(str_v);
    const char *old_s = x_as_str(old_v);
    const char *new_s = x_as_str(new_v);
    size_t old_len = strlen(old_s);
    size_t new_len = strlen(new_s);
    if (old_len == 0) return str_v;
    size_t count = 0;
    const char *p = s;
    while ((p = strstr(p, old_s)) != NULL) { count++; p += old_len; }
    size_t result_len = strlen(s) + count * (new_len - old_len);
    char *buf = (char *)malloc(result_len + 1);
    char *out = buf;
    p = s;
    const char *found;
    while ((found = strstr(p, old_s)) != NULL) {
        size_t seg = found - p;
        memcpy(out, p, seg); out += seg;
        memcpy(out, new_s, new_len); out += new_len;
        p = found + old_len;
    }
    strcpy(out, p);
    return x_string_own(buf);
}

/* ========== Array builtins ========== */

static inline XValue x_new_array(XValue size, XValue init) {
    int64_t n = x_as_int(size);
    XArray *a = x_array_new(n);
    for (int64_t i = 0; i < n; i++) {
        x_array_push(a, init);
    }
    return x_array_val(a);
}

static inline XValue x_push(XValue arr_v, XValue val) {
    if (arr_v.tag != X_ARRAY) return arr_v;
    x_array_push(arr_v.as.arr, val);
    return arr_v;
}

static inline XValue x_copy_array(XValue arr_v) {
    if (arr_v.tag != X_ARRAY) return arr_v;
    XArray *src = arr_v.as.arr;
    XArray *dst = x_array_new(src->length);
    for (int64_t i = 0; i < src->length; i++) {
        x_array_push(dst, src->items[i]);
    }
    return x_array_val(dst);
}

static inline XValue x_swap(XValue arr_v, XValue i_v, XValue j_v) {
    if (arr_v.tag != X_ARRAY) return x_null();
    int64_t i = x_as_int(i_v);
    int64_t j = x_as_int(j_v);
    XArray *a = arr_v.as.arr;
    XValue tmp = a->items[i];
    a->items[i] = a->items[j];
    a->items[j] = tmp;
    return x_null();
}

static int x_sort_cmp_desc(const void *a, const void *b) {
    const XValue *va = (const XValue *)a;
    const XValue *vb = (const XValue *)b;
    if (va->tag != X_ARRAY || vb->tag != X_ARRAY) return 0;
    XValue ca = va->as.arr->items[1];
    XValue cb = vb->as.arr->items[1];
    double da = x_as_float(ca), db = x_as_float(cb);
    if (db > da) return 1;
    if (db < da) return -1;
    return 0;
}

static inline XValue x_sort_by_value_desc(XValue arr_v) {
    if (arr_v.tag != X_ARRAY) return x_null();
    XArray *a = arr_v.as.arr;
    qsort(a->items, a->length, sizeof(XValue), x_sort_cmp_desc);
    return x_null();
}

// JSON序列化函数
static void x_json_serialize(XValue v, char *buffer, size_t *size, size_t capacity) {
    switch (v.tag) {
        case X_INT:
            snprintf(buffer + *size, capacity - *size, "%lld", (long long)v.as.i);
            *size += strlen(buffer + *size);
            break;
        case X_FLOAT:
            snprintf(buffer + *size, capacity - *size, "%g", v.as.f);
            *size += strlen(buffer + *size);
            break;
        case X_BOOL:
            snprintf(buffer + *size, capacity - *size, "%s", v.as.b ? "true" : "false");
            *size += strlen(buffer + *size);
            break;
        case X_STRING:
            snprintf(buffer + *size, capacity - *size, "\"%s\"", v.as.s);
            *size += strlen(buffer + *size);
            break;
        case X_ARRAY:
            strcat(buffer + *size, "[");
            *size += 1;
            for (int64_t i = 0; i < v.as.arr->length; i++) {
                if (i > 0) {
                    strcat(buffer + *size, ",");
                    *size += 1;
                }
                x_json_serialize(v.as.arr->items[i], buffer, size, capacity);
            }
            strcat(buffer + *size, "]");
            *size += 1;
            break;
        case X_MAP:
            strcat(buffer + *size, "{");
            *size += 1;
            bool first = true;
            for (int64_t i = 0; i < v.as.map->num_buckets; i++) {
                XMapEntry *e = v.as.map->buckets[i];
                while (e) {
                    if (!first) {
                        strcat(buffer + *size, ",");
                        *size += 1;
                    }
                    first = false;
                    snprintf(buffer + *size, capacity - *size, "\"%s\":", e->key);
                    *size += strlen(buffer + *size);
                    x_json_serialize(e->value, buffer, size, capacity);
                    e = e->next;
                }
            }
            strcat(buffer + *size, "}");
            *size += 1;
            break;
        case X_NULL:
            strcat(buffer + *size, "null");
            *size += 4;
            break;
        case X_NONE:
            strcat(buffer + *size, "null");
            *size += 4;
            break;
    }
}

static inline XValue x_to_json(XValue v) {
    char buffer[4096];
    size_t size = 0;
    memset(buffer, 0, sizeof(buffer));
    x_json_serialize(v, buffer, &size, sizeof(buffer));
    return x_string(buffer);
}

// 简单的JSON解析函数（仅支持基本类型）
static XValue x_json_parse_internal(const char *json, size_t *pos);

static XValue x_json_parse_string(const char *json, size_t *pos) {
    (*pos)++; // 跳过开始引号
    const char *start = json + *pos;
    while (json[*pos] != '"' && json[*pos] != '\0') {
        if (json[*pos] == '\\') {
            (*pos)++;
        }
        (*pos)++;
    }
    size_t len = *pos - (start - json);
    char *str = (char *)malloc(len + 1);
    memcpy(str, start, len);
    str[len] = '\0';
    (*pos)++; // 跳过结束引号
    return x_string_own(str);
}

static XValue x_json_parse_array(const char *json, size_t *pos) {
    (*pos)++; // 跳过开始括号
    XArray *arr = x_array_new(8);
    while (json[*pos] != ']' && json[*pos] != '\0') {
        XValue elem = x_json_parse_internal(json, pos);
        x_array_push(arr, elem);
        if (json[*pos] == ',') {
            (*pos)++;
        }
    }
    (*pos)++; // 跳过结束括号
    return x_array_val(arr);
}

static XValue x_json_parse_object(const char *json, size_t *pos) {
    (*pos)++; // 跳过开始大括号
    XMap *map = x_map_new();
    while (json[*pos] != '}' && json[*pos] != '\0') {
        // 解析键
        XValue key = x_json_parse_string(json, pos);
        if (json[*pos] == ':') {
            (*pos)++;
            // 解析值
            XValue value = x_json_parse_internal(json, pos);
            x_map_set(map, x_as_str(key), value);
        }
        if (json[*pos] == ',') {
            (*pos)++;
        }
    }
    (*pos)++; // 跳过结束大括号
    return x_map_val(map);
}

static XValue x_json_parse_internal(const char *json, size_t *pos) {
    // 跳过空白字符
    while (json[*pos] == ' ' || json[*pos] == '\t' || json[*pos] == '\n' || json[*pos] == '\r') {
        (*pos)++;
    }
    
    switch (json[*pos]) {
        case '"':
            return x_json_parse_string(json, pos);
        case '[':
            return x_json_parse_array(json, pos);
        case '{':
            return x_json_parse_object(json, pos);
        case 't':
            if (strncmp(json + *pos, "true", 4) == 0) {
                *pos += 4;
                return x_bool(true);
            }
            break;
        case 'f':
            if (strncmp(json + *pos, "false", 5) == 0) {
                *pos += 5;
                return x_bool(false);
            }
            break;
        case 'n':
            if (strncmp(json + *pos, "null", 4) == 0) {
                *pos += 4;
                return x_null();
            }
            break;
        case '-':
        case '0': case '1': case '2': case '3': case '4':
        case '5': case '6': case '7': case '8': case '9': {
            char *end;
            double num = strtod(json + *pos, &end);
            *pos = end - json;
            if (num == (int64_t)num) {
                return x_int((int64_t)num);
            } else {
                return x_float(num);
            }
        }
    }
    return x_null();
}

static inline XValue x_json_parse(XValue json_str) {
    const char *json = x_as_str(json_str);
    size_t pos = 0;
    return x_json_parse_internal(json, &pos);
}

/* ========== Math builtins ========== */

static inline XValue x_sqrt(XValue v) { return x_float(sqrt(x_as_float(v))); }
static inline XValue x_pow(XValue base, XValue exp) { return x_float(pow(x_as_float(base), x_as_float(exp))); }
static inline XValue x_abs(XValue v) { return x_float(fabs(x_as_float(v))); }
static inline XValue x_sin(XValue v) { return x_float(sin(x_as_float(v))); }
static inline XValue x_cos(XValue v) { return x_float(cos(x_as_float(v))); }
static inline XValue x_floor(XValue v) { return x_float(floor(x_as_float(v))); }
static inline XValue x_ceil(XValue v) { return x_float(ceil(x_as_float(v))); }
static inline XValue x_round(XValue v) { return x_float(round(x_as_float(v))); }

/* ========== Type conversions ========== */

static inline XValue x_to_int(XValue v) { return x_int(x_as_int(v)); }
static inline XValue x_to_float(XValue v) { return x_float(x_as_float(v)); }

static inline XValue x_format_float(XValue v, XValue prec) {
    char buf[64];
    snprintf(buf, sizeof(buf), "%.*f", (int)x_as_int(prec), x_as_float(v));
    return x_string(buf);
}

static inline XValue x_type_of(XValue v) {
    switch (v.tag) {
        case X_INT: return x_string("int");
        case X_FLOAT: return x_string("float");
        case X_BOOL: return x_string("bool");
        case X_STRING: return x_string("string");
        case X_ARRAY: return x_string("array");
        case X_MAP: return x_string("map");
        case X_NULL: return x_string("null");
        case X_NONE: return x_string("none");
    }
    return x_string("unknown");
}

/* ========== Arithmetic on XValue ========== */

static inline XValue x_add(XValue a, XValue b) {
    if (a.tag == X_STRING || b.tag == X_STRING) return x_concat(a.tag == X_STRING ? a : x_to_string(a), b.tag == X_STRING ? b : x_to_string(b));
    if (a.tag == X_FLOAT || b.tag == X_FLOAT) return x_float(x_as_float(a) + x_as_float(b));
    return x_int(a.as.i + b.as.i);
}

static inline XValue x_sub(XValue a, XValue b) {
    if (a.tag == X_FLOAT || b.tag == X_FLOAT) return x_float(x_as_float(a) - x_as_float(b));
    return x_int(a.as.i - b.as.i);
}

static inline XValue x_mul(XValue a, XValue b) {
    if (a.tag == X_FLOAT || b.tag == X_FLOAT) return x_float(x_as_float(a) * x_as_float(b));
    return x_int(a.as.i * b.as.i);
}

static inline XValue x_div(XValue a, XValue b) {
    if (a.tag == X_FLOAT || b.tag == X_FLOAT) return x_float(x_as_float(a) / x_as_float(b));
    int64_t bv = b.as.i;
    if (bv == 0) { fprintf(stderr, "Division by zero\n"); exit(1); }
    return x_int(a.as.i / bv);
}

static inline XValue x_mod(XValue a, XValue b) {
    if (a.tag == X_FLOAT || b.tag == X_FLOAT) return x_float(fmod(x_as_float(a), x_as_float(b)));
    int64_t bv = b.as.i;
    if (bv == 0) { fprintf(stderr, "Modulo by zero\n"); exit(1); }
    return x_int(a.as.i % bv);
}

static inline XValue x_neg(XValue a) {
    if (a.tag == X_FLOAT) return x_float(-a.as.f);
    return x_int(-a.as.i);
}

static inline XValue x_not(XValue a) { return x_bool(!x_as_bool(a)); }

/* ========== Comparison ========== */

static inline XValue x_eq(XValue a, XValue b) {
    if (a.tag == X_STRING && b.tag == X_STRING) return x_bool(strcmp(a.as.s, b.as.s) == 0);
    if (a.tag == X_INT && b.tag == X_INT) return x_bool(a.as.i == b.as.i);
    if (a.tag == X_FLOAT || b.tag == X_FLOAT) return x_bool(x_as_float(a) == x_as_float(b));
    if (a.tag == X_BOOL && b.tag == X_BOOL) return x_bool(a.as.b == b.as.b);
    if (a.tag == X_NULL && b.tag == X_NULL) return x_bool(true);
    if (a.tag == X_NONE && b.tag == X_NONE) return x_bool(true);
    return x_bool(false);
}

static inline XValue x_neq(XValue a, XValue b) { return x_bool(!x_as_bool(x_eq(a, b))); }

static inline XValue x_lt(XValue a, XValue b) {
    if (a.tag == X_FLOAT || b.tag == X_FLOAT) return x_bool(x_as_float(a) < x_as_float(b));
    return x_bool(a.as.i < b.as.i);
}

static inline XValue x_le(XValue a, XValue b) {
    if (a.tag == X_FLOAT || b.tag == X_FLOAT) return x_bool(x_as_float(a) <= x_as_float(b));
    return x_bool(a.as.i <= b.as.i);
}

static inline XValue x_gt(XValue a, XValue b) {
    if (a.tag == X_FLOAT || b.tag == X_FLOAT) return x_bool(x_as_float(a) > x_as_float(b));
    return x_bool(a.as.i > b.as.i);
}

static inline XValue x_ge(XValue a, XValue b) {
    if (a.tag == X_FLOAT || b.tag == X_FLOAT) return x_bool(x_as_float(a) >= x_as_float(b));
    return x_bool(a.as.i >= b.as.i);
}

static inline XValue x_and(XValue a, XValue b) { return x_bool(x_as_bool(a) && x_as_bool(b)); }
static inline XValue x_or(XValue a, XValue b) { return x_bool(x_as_bool(a) || x_as_bool(b)); }

/* ========== Index operations ========== */

static inline XValue x_index_get(XValue obj, XValue idx) {
    if (obj.tag == X_ARRAY) return x_array_get(obj.as.arr, x_as_int(idx));
    if (obj.tag == X_STRING) return x_char_at(obj, idx);
    if (obj.tag == X_MAP && idx.tag == X_STRING) return x_map_get(obj.as.map, idx.as.s);
    fprintf(stderr, "Cannot index value of type %d\n", obj.tag);
    exit(1);
}

static inline void x_index_set(XValue obj, XValue idx, XValue val) {
    if (obj.tag == X_ARRAY) { x_array_set(obj.as.arr, x_as_int(idx), val); return; }
    if (obj.tag == X_MAP && idx.tag == X_STRING) { x_map_set(obj.as.map, idx.as.s, val); return; }
    fprintf(stderr, "Cannot set index on value of type %d\n", obj.tag);
    exit(1);
}

/* ========== Regex (simple pattern matcher) ========== */

static int x_simple_match_at(const char *text, int tpos, int tlen,
                              const char *pat, int ppos, int plen);

static int x_simple_match_at(const char *text, int tpos, int tlen,
                              const char *pat, int ppos, int plen) {
    while (ppos < plen && tpos <= tlen) {
        if (pat[ppos] == '[') {
            int end = ppos + 1;
            while (end < plen && pat[end] != ']') end++;
            if (tpos >= tlen) return 0;
            bool found = false;
            for (int k = ppos + 1; k < end; k++) {
                if (pat[k] == text[tpos]) { found = true; break; }
            }
            if (!found) return 0;
            ppos = end + 1;
            tpos++;
        } else if (pat[ppos] == '.') {
            if (tpos >= tlen) return 0;
            ppos++; tpos++;
        } else {
            if (tpos >= tlen || pat[ppos] != text[tpos]) return 0;
            ppos++; tpos++;
        }
    }
    return ppos >= plen ? 1 : 0;
}

static inline int64_t x_regex_count_branch(const char *text, int tlen,
                                            const char *pat, int plen) {
    int64_t count = 0;
    for (int i = 0; i <= tlen - plen + 10; i++) {
        if (i >= tlen) break;
        if (x_simple_match_at(text, i, tlen, pat, 0, plen)) count++;
    }
    return count;
}

static inline XValue x_regex_match_count(XValue text_v, XValue pattern_v) {
    const char *text = x_as_str(text_v);
    const char *pat = x_as_str(pattern_v);
    int tlen = (int)strlen(text);

    int64_t total = 0;
    const char *p = pat;
    while (*p) {
        const char *bar = strchr(p, '|');
        int blen;
        if (bar) {
            blen = (int)(bar - p);
        } else {
            blen = (int)strlen(p);
        }
        char *branch = (char *)malloc(blen + 1);
        memcpy(branch, p, blen);
        branch[blen] = '\0';
        total += x_regex_count_branch(text, tlen, branch, blen);
        free(branch);
        if (bar) p = bar + 1; else break;
    }
    return x_int(total);
}

/* ========== Pi digits (Gibbons' spigot with simple bignum) ========== */

/* Simple arbitrary-precision integer for pi computation */
typedef struct { uint32_t *d; int len; int cap; int sign; } xbn;

static xbn xbn_new(int cap) { xbn b; b.cap=cap<4?4:cap; b.len=0; b.sign=1; b.d=(uint32_t*)calloc(b.cap,sizeof(uint32_t)); return b; }
static void xbn_free(xbn *b) { free(b->d); b->d=NULL; b->len=0; }
static void xbn_ensure(xbn *b, int n) { if(n>b->cap){b->cap=n*2;b->d=(uint32_t*)realloc(b->d,b->cap*sizeof(uint32_t));} }
static xbn xbn_from(int64_t v) {
    xbn b=xbn_new(4);
    if(v<0){b.sign=-1;v=-v;}
    if(v==0){b.len=1;b.d[0]=0;return b;}
    while(v>0){xbn_ensure(&b,b.len+1);b.d[b.len++]=(uint32_t)(v&0xFFFFFFFF);v>>=32;}
    return b;
}
static xbn xbn_copy(xbn *a) { xbn b=xbn_new(a->len); b.len=a->len; b.sign=a->sign; memcpy(b.d,a->d,a->len*sizeof(uint32_t)); return b; }

static xbn xbn_add_abs(xbn *a, xbn *b) {
    int n=a->len>b->len?a->len:b->len;
    xbn r=xbn_new(n+1); r.len=n+1;
    uint64_t carry=0;
    for(int i=0;i<n+1;i++){
        uint64_t va=i<a->len?(uint64_t)a->d[i]:0;
        uint64_t vb=i<b->len?(uint64_t)b->d[i]:0;
        uint64_t s=va+vb+carry; r.d[i]=(uint32_t)(s&0xFFFFFFFF); carry=s>>32;
    }
    while(r.len>1&&r.d[r.len-1]==0)r.len--;
    return r;
}

static int xbn_cmp_abs(xbn *a, xbn *b) {
    if(a->len!=b->len) return a->len>b->len?1:-1;
    for(int i=a->len-1;i>=0;i--) { if(a->d[i]!=b->d[i]) return a->d[i]>b->d[i]?1:-1; }
    return 0;
}

static xbn xbn_sub_abs(xbn *a, xbn *b) {
    xbn r=xbn_new(a->len); r.len=a->len;
    int64_t borrow=0;
    for(int i=0;i<a->len;i++){
        int64_t va=(int64_t)a->d[i];
        int64_t vb=i<b->len?(int64_t)b->d[i]:0;
        int64_t d=va-vb-borrow;
        if(d<0){d+=(int64_t)1<<32;borrow=1;}else{borrow=0;}
        r.d[i]=(uint32_t)d;
    }
    while(r.len>1&&r.d[r.len-1]==0)r.len--;
    return r;
}

static xbn xbn_mul_small(xbn *a, uint64_t v) {
    xbn r=xbn_new(a->len+2); r.len=a->len+2; r.sign=a->sign;
    uint64_t carry=0;
    for(int i=0;i<a->len;i++){
        uint64_t p=(uint64_t)a->d[i]*v+carry;
        r.d[i]=(uint32_t)(p&0xFFFFFFFF); carry=p>>32;
    }
    r.d[a->len]=(uint32_t)(carry&0xFFFFFFFF);
    r.d[a->len+1]=(uint32_t)(carry>>32);
    while(r.len>1&&r.d[r.len-1]==0)r.len--;
    return r;
}

static xbn xbn_add(xbn *a, xbn *b) {
    if(a->sign==b->sign){ xbn r=xbn_add_abs(a,b); r.sign=a->sign; return r; }
    int c=xbn_cmp_abs(a,b);
    if(c==0) return xbn_from(0);
    if(c>0){ xbn r=xbn_sub_abs(a,b); r.sign=a->sign; return r; }
    xbn r=xbn_sub_abs(b,a); r.sign=b->sign; return r;
}

static xbn xbn_mul(xbn *a, xbn *b) {
    int n=a->len+b->len;
    xbn r=xbn_new(n); r.len=n; r.sign=a->sign*b->sign;
    for(int i=0;i<a->len;i++){
        uint64_t carry=0;
        for(int j=0;j<b->len;j++){
            uint64_t p=(uint64_t)a->d[i]*(uint64_t)b->d[j]+(uint64_t)r.d[i+j]+carry;
            r.d[i+j]=(uint32_t)(p&0xFFFFFFFF); carry=p>>32;
        }
        r.d[i+b->len]+=(uint32_t)carry;
    }
    while(r.len>1&&r.d[r.len-1]==0)r.len--;
    return r;
}

static xbn xbn_div(xbn *num, xbn *den, xbn *rem) {
    if(den->len==1&&den->d[0]==0){*rem=xbn_from(0);return xbn_from(0);}
    if(xbn_cmp_abs(num,den)<0){if(rem)*rem=xbn_copy(num);return xbn_from(0);}
    if(den->len==1){
        uint64_t dv=den->d[0]; uint64_t carry=0;
        xbn q=xbn_new(num->len); q.len=num->len; q.sign=num->sign*den->sign;
        for(int i=num->len-1;i>=0;i--){
            uint64_t cur=carry*(((uint64_t)1)<<32)+(uint64_t)num->d[i];
            q.d[i]=(uint32_t)(cur/dv); carry=cur%dv;
        }
        while(q.len>1&&q.d[q.len-1]==0)q.len--;
        if(rem)*rem=xbn_from((int64_t)carry);
        return q;
    }
    /* For multi-word division, use repeated subtraction (slow but correct for small numbers) */
    xbn a=xbn_copy(num); a.sign=1;
    xbn b=xbn_copy(den); b.sign=1;
    xbn q=xbn_from(0);
    while(xbn_cmp_abs(&a,&b)>=0){
        xbn t=xbn_sub_abs(&a,&b);
        xbn_free(&a); a=t;
        xbn one=xbn_from(1);
        xbn nq=xbn_add(&q,&one);
        xbn_free(&q); q=nq;
        xbn_free(&one);
    }
    q.sign=num->sign*den->sign;
    if(rem){*rem=a;}else{xbn_free(&a);}
    xbn_free(&b);
    return q;
}

static inline XValue x_compute_pi_digits(XValue n_v) {
    int n = (int)x_as_int(n_v);
    char *result = (char *)malloc(n + 2);
    int result_len = 0;

    xbn q=xbn_from(1), r=xbn_from(0), t=xbn_from(1);
    int64_t k=0;

    while (result_len < n) {
        k++;
        int64_t l = 2*k+1;
        /* compose: q,r,t = q*k, (2*q+r)*l, t*l */
        xbn qk=xbn_mul_small(&q,(uint64_t)k);
        xbn twoq=xbn_mul_small(&q,2);
        xbn twoq_r=xbn_add(&twoq,&r);
        xbn nr=xbn_mul_small(&twoq_r,(uint64_t)l);
        xbn nt=xbn_mul_small(&t,(uint64_t)l);
        xbn_free(&twoq); xbn_free(&twoq_r);
        xbn_free(&q); xbn_free(&r); xbn_free(&t);
        q=qk; r=nr; t=nt;

        /* extract: digit = (3*q+r)/t vs (4*q+r)/t */
        xbn q3=xbn_mul_small(&q,3);
        xbn q3r=xbn_add(&q3,&r);
        xbn rem1;
        xbn d1=xbn_div(&q3r,&t,&rem1);

        xbn q4=xbn_mul_small(&q,4);
        xbn q4r=xbn_add(&q4,&r);
        xbn rem2;
        xbn d2=xbn_div(&q4r,&t,&rem2);

        int64_t digit1 = (d1.len==1) ? (int64_t)d1.d[0]*d1.sign : -1;
        int64_t digit2 = (d2.len==1) ? (int64_t)d2.d[0]*d2.sign : -1;

        xbn_free(&q3); xbn_free(&q3r); xbn_free(&rem1);
        xbn_free(&q4); xbn_free(&q4r); xbn_free(&rem2);

        if (digit1 == digit2 && digit1 >= 0 && digit1 <= 9) {
            result[result_len++] = '0' + (int)digit1;
            /* extract: q,r,t = 10*q, 10*(r - digit*t), t */
            xbn dt=xbn_mul_small(&t,(uint64_t)digit1);
            dt.sign = -dt.sign;
            xbn rdiff=xbn_add(&r,&dt);
            xbn new_r=xbn_mul_small(&rdiff,10);
            xbn new_q=xbn_mul_small(&q,10);
            xbn_free(&dt); xbn_free(&rdiff);
            xbn_free(&q); xbn_free(&r);
            q=new_q; r=new_r;
        }
        xbn_free(&d1); xbn_free(&d2);
    }
    xbn_free(&q); xbn_free(&r); xbn_free(&t);
    result[result_len] = '\0';
    return x_string_own(result);
}

/* ========== Map builtins (wrapped) ========== */

static inline XValue x_builtin_new_map(void) { return x_map_val(x_map_new()); }

static inline XValue x_builtin_map_set(XValue map_v, XValue key_v, XValue val) {
    if (map_v.tag != X_MAP) return x_null();
    x_map_set(map_v.as.map, x_as_str(key_v), val);
    return x_null();
}

static inline XValue x_builtin_map_get(XValue map_v, XValue key_v) {
    if (map_v.tag != X_MAP) return x_int(0);
    return x_map_get(map_v.as.map, x_as_str(key_v));
}

static inline XValue x_builtin_map_contains(XValue map_v, XValue key_v) {
    if (map_v.tag != X_MAP) return x_bool(false);
    return x_bool(x_map_contains(map_v.as.map, x_as_str(key_v)));
}

static inline XValue x_builtin_map_keys(XValue map_v) {
    if (map_v.tag != X_MAP) return x_array_val(x_array_new(0));
    return x_map_keys(map_v.as.map);
}

/* ========== System functions ========== */

// 环境变量相关函数
static inline XValue x_get_env(XValue name_v) {
    const char *name = x_as_str(name_v);
    const char *value = getenv(name);
    if (value) {
        return x_string(value);
    } else {
        return x_string("");
    }
}

static inline XValue x_set_env(XValue name_v, XValue value_v) {
    const char *name = x_as_str(name_v);
    const char *value = x_as_str(value_v);
    int result = setenv(name, value, 1);
    return x_bool(result == 0);
}

static inline XValue x_unset_env(XValue name_v) {
    const char *name = x_as_str(name_v);
    int result = unsetenv(name);
    return x_bool(result == 0);
}

static inline XValue x_env_vars(void) {
    XArray *arr = x_array_new(10);
#ifdef _WIN32
    char *env = GetEnvironmentStrings();
    char *p = env;
    while (*p) {
        if (*p != '=') {
            char *key_start = p;
            while (*p && *p != '=') p++;
            if (*p == '=') {
                *p = '\0';
                char *key = key_start;
                char *value = p + 1;
                XValue key_val = x_string(key);
                XValue value_val = x_string(value);
                XArray *pair = x_array_new(2);
                x_array_push(pair, key_val);
                x_array_push(pair, value_val);
                x_array_push(arr, x_array_val(pair));
            }
        }
        while (*p) p++;
        p++;
    }
    FreeEnvironmentStrings(env);
#else
    extern char **environ;
    char **env = environ;
    while (*env) {
        char *p = *env;
        char *key_start = p;
        while (*p && *p != '=') p++;
        if (*p == '=') {
            *p = '\0';
            char *key = key_start;
            char *value = p + 1;
            XValue key_val = x_string(key);
            XValue value_val = x_string(value);
            XArray *pair = x_array_new(2);
            x_array_push(pair, key_val);
            x_array_push(pair, value_val);
            x_array_push(arr, x_array_val(pair));
        }
        env++;
    }
#endif
    return x_array_val(arr);
}

// 进程操作相关函数
static inline XValue x_getpid(void) {
#ifdef _WIN32
    return x_int((int64_t)GetCurrentProcessId());
#else
    return x_int((int64_t)getpid());
#endif
}

static inline XValue x_getppid(void) {
#ifdef _WIN32
    return x_int((int64_t)GetCurrentProcessId() - 1); // 模拟
#else
    return x_int((int64_t)getppid());
#endif
}

static inline void x_exit(XValue code_v) {
    int code = (int)x_as_int(code_v);
    exit(code);
}

static inline XValue x_system(XValue command_v) {
    const char *command = x_as_str(command_v);
    int result = system(command);
    return x_int((int64_t)result);
}

static inline XValue x_command_output(XValue command_v) {
    const char *command = x_as_str(command_v);
    char buffer[1024];
    char *output = NULL;
    size_t output_size = 0;
    size_t output_capacity = 1024;
    
    output = (char *)malloc(output_capacity);
    if (!output) {
        return x_string("");
    }
    output[0] = '\0';
    
#ifdef _WIN32
    FILE *pipe = _popen(command, "r");
#else
    FILE *pipe = popen(command, "r");
#endif
    
    if (pipe) {
        while (fgets(buffer, sizeof(buffer), pipe) != NULL) {
            size_t len = strlen(buffer);
            if (output_size + len + 1 > output_capacity) {
                output_capacity *= 2;
                char *new_output = (char *)realloc(output, output_capacity);
                if (!new_output) {
                    free(output);
#ifdef _WIN32
                    _pclose(pipe);
#else
                    pclose(pipe);
#endif
                    return x_string("");
                }
                output = new_output;
            }
            strcat(output, buffer);
            output_size += len;
        }
#ifdef _WIN32
        _pclose(pipe);
#else
        pclose(pipe);
#endif
    }
    
    XValue result = x_string_own(output);
    return result;
}

// 系统信息相关函数
static inline XValue x_os_type(void) {
#ifdef _WIN32
    return x_string("Windows");
#else
    struct utsname info;
    if (uname(&info) == 0) {
        return x_string(info.sysname);
    } else {
        return x_string("Unknown");
    }
#endif
}

static inline XValue x_os_version(void) {
#ifdef _WIN32
    OSVERSIONINFOEX info;
    ZeroMemory(&info, sizeof(info));
    info.dwOSVersionInfoSize = sizeof(info);
    if (GetVersionEx((OSVERSIONINFO*)&info)) {
        char buffer[64];
        snprintf(buffer, sizeof(buffer), "%d.%d.%d", info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber);
        return x_string(buffer);
    } else {
        return x_string("Unknown");
    }
#else
    struct utsname info;
    if (uname(&info) == 0) {
        return x_string(info.release);
    } else {
        return x_string("Unknown");
    }
#endif
}

static inline XValue x_hostname(void) {
#ifdef _WIN32
    char buffer[256];
    if (GetComputerName(buffer, (DWORD*)sizeof(buffer))) {
        return x_string(buffer);
    } else {
        return x_string("Unknown");
    }
#else
    char buffer[256];
    if (gethostname(buffer, sizeof(buffer)) == 0) {
        return x_string(buffer);
    } else {
        return x_string("Unknown");
    }
#endif
}

static inline XValue x_arch(void) {
#ifdef _WIN32
    SYSTEM_INFO info;
    GetSystemInfo(&info);
    if (info.wProcessorArchitecture == PROCESSOR_ARCHITECTURE_AMD64) {
        return x_string("x86_64");
    } else if (info.wProcessorArchitecture == PROCESSOR_ARCHITECTURE_INTEL) {
        return x_string("x86");
    } else {
        return x_string("Unknown");
    }
#else
    struct utsname info;
    if (uname(&info) == 0) {
        return x_string(info.machine);
    } else {
        return x_string("Unknown");
    }
#endif
}

static inline XValue x_free_memory(void) {
#ifdef _WIN32
    MEMORYSTATUSEX info;
    info.dwLength = sizeof(info);
    if (GlobalMemoryStatusEx(&info)) {
        return x_int((int64_t)info.ullAvailPhys);
    } else {
        return x_int(0);
    }
#else
    struct sysinfo info;
    if (sysinfo(&info) == 0) {
        return x_int((int64_t)info.freeram * info.mem_unit);
    } else {
        return x_int(0);
    }
#endif
}

static inline XValue x_total_memory(void) {
#ifdef _WIN32
    MEMORYSTATUSEX info;
    info.dwLength = sizeof(info);
    if (GlobalMemoryStatusEx(&info)) {
        return x_int((int64_t)info.ullTotalPhys);
    } else {
        return x_int(0);
    }
#else
    struct sysinfo info;
    if (sysinfo(&info) == 0) {
        return x_int((int64_t)info.totalram * info.mem_unit);
    } else {
        return x_int(0);
    }
#endif
}

static inline XValue x_cpu_count(void) {
#ifdef _WIN32
    SYSTEM_INFO info;
    GetSystemInfo(&info);
    return x_int((int64_t)info.dwNumberOfProcessors);
#else
    return x_int((int64_t)sysconf(_SC_NPROCESSORS_ONLN));
#endif
}

// 路径操作相关函数
static inline XValue x_current_dir(void) {
    char buffer[1024];
#ifdef _WIN32
    if (GetCurrentDirectory(sizeof(buffer), buffer)) {
        return x_string(buffer);
    } else {
        return x_string(".");
    }
#else
    if (getcwd(buffer, sizeof(buffer))) {
        return x_string(buffer);
    } else {
        return x_string(".");
    }
#endif
}

static inline XValue x_chdir(XValue path_v) {
    const char *path = x_as_str(path_v);
#ifdef _WIN32
    int result = SetCurrentDirectory(path);
    return x_bool(result != 0);
#else
    int result = chdir(path);
    return x_bool(result == 0);
#endif
}

static inline XValue x_path_dirname(XValue path_v) {
    const char *path = x_as_str(path_v);
    char *copy = strdup(path);
    if (!copy) {
        return x_string(".");
    }
    char *last_sep = NULL;
#ifdef _WIN32
    last_sep = strrchr(copy, '\\');
    if (!last_sep) {
        last_sep = strrchr(copy, '/');
    }
#else
    last_sep = strrchr(copy, '/');
#endif
    if (last_sep) {
        *last_sep = '\0';
        XValue result = x_string(copy);
        free(copy);
        return result;
    } else {
        free(copy);
        return x_string(".");
    }
}

static inline XValue x_path_basename(XValue path_v) {
    const char *path = x_as_str(path_v);
    char *last_sep = NULL;
#ifdef _WIN32
    last_sep = strrchr(path, '\\');
    if (!last_sep) {
        last_sep = strrchr(path, '/');
    }
#else
    last_sep = strrchr(path, '/');
#endif
    if (last_sep) {
        return x_string(last_sep + 1);
    } else {
        return x_string(path);
    }
}

static inline XValue x_path_extension(XValue path_v) {
    const char *path = x_as_str(path_v);
    char *last_dot = strrchr(path, '.');
    char *last_sep = NULL;
#ifdef _WIN32
    last_sep = strrchr(path, '\\');
    if (!last_sep) {
        last_sep = strrchr(path, '/');
    }
#else
    last_sep = strrchr(path, '/');
#endif
    if (last_dot && (!last_sep || last_dot > last_sep)) {
        return x_string(last_dot + 1);
    } else {
        return x_string("");
    }
}

static inline XValue x_path_exists(XValue path_v) {
    const char *path = x_as_str(path_v);
#ifdef _WIN32
    DWORD attributes = GetFileAttributes(path);
    return x_bool(attributes != INVALID_FILE_ATTRIBUTES);
#else
    struct stat st;
    int result = stat(path, &st);
    return x_bool(result == 0);
#endif
}

static inline XValue x_is_file(XValue path_v) {
    const char *path = x_as_str(path_v);
#ifdef _WIN32
    DWORD attributes = GetFileAttributes(path);
    return x_bool(attributes != INVALID_FILE_ATTRIBUTES && !(attributes & FILE_ATTRIBUTE_DIRECTORY));
#else
    struct stat st;
    int result = stat(path, &st);
    return x_bool(result == 0 && S_ISREG(st.st_mode));
#endif
}

static inline XValue x_is_dir(XValue path_v) {
    const char *path = x_as_str(path_v);
#ifdef _WIN32
    DWORD attributes = GetFileAttributes(path);
    return x_bool(attributes != INVALID_FILE_ATTRIBUTES && (attributes & FILE_ATTRIBUTE_DIRECTORY));
#else
    struct stat st;
    int result = stat(path, &st);
    return x_bool(result == 0 && S_ISDIR(st.st_mode));
#endif
}

// 临时文件相关函数
static inline XValue x_temp_file(void) {
    char buffer[1024];
#ifdef _WIN32
    if (GetTempFileName(getenv("TEMP"), "x", 0, buffer)) {
        return x_string(buffer);
    } else {
        return x_string("");
    }
#else
    strcpy(buffer, "/tmp/x_XXXXXX");
    if (mkstemp(buffer) != -1) {
        return x_string(buffer);
    } else {
        return x_string("");
    }
#endif
}

static inline XValue x_temp_dir(void) {
    char buffer[1024];
#ifdef _WIN32
    if (GetTempPath(sizeof(buffer), buffer)) {
        char subdir[1024];
        snprintf(subdir, sizeof(subdir), "%s\\x_XXXXXX", buffer);
        if (GetTempFileName(buffer, "x", 0, subdir)) {
            if (CreateDirectory(subdir, NULL)) {
                return x_string(subdir);
            }
        }
    }
    return x_string("");
#else
    strcpy(buffer, "/tmp/x_XXXXXX");
    if (mkdtemp(buffer)) {
        return x_string(buffer);
    } else {
        return x_string("");
    }
#endif
}

static inline XValue x_get_temp_dir(void) {
    char buffer[1024];
#ifdef _WIN32
    if (GetTempPath(sizeof(buffer), buffer)) {
        return x_string(buffer);
    } else {
        return x_string("C:\\Temp");
    }
#else
    const char *temp_dir = getenv("TMPDIR");
    if (temp_dir) {
        return x_string(temp_dir);
    } else {
        return x_string("/tmp");
    }
#endif
}

// 信号处理相关函数
static inline XValue x_signal(XValue signum_v, XValue handler_v) {
    // 简单实现，实际需要更复杂的处理
    return x_bool(true);
}

static inline XValue x_kill(XValue pid_v, XValue signum_v) {
    int pid = (int)x_as_int(pid_v);
    int signum = (int)x_as_int(signum_v);
#ifdef _WIN32
    BOOL result = TerminateProcess((HANDLE)pid, signum);
    return x_bool(result != 0);
#else
    int result = kill(pid, signum);
    return x_bool(result == 0);
#endif
}

// 线程相关函数
static DWORD WINAPI x_thread_start(LPVOID lpParam) {
    // 简单实现，实际需要调用X语言函数
    return 0;
}

// 其他函数
static inline XValue x_syscall(XValue number_v, XValue args_v) {
    int64_t number = x_as_int(number_v);
    
    if (args_v.tag != X_ARRAY) {
        return x_int(-1);
    }
    
    XArray *args = args_v.as.arr;
    int64_t arg_count = args->length;
    
    // 简化的网络系统调用实现
    switch (number) {
        case 1: { // SYS_SOCKET
            if (arg_count < 3) return x_int(-1);
            int domain = (int)x_as_int(args->items[0]);
            int type = (int)x_as_int(args->items[1]);
            int protocol = (int)x_as_int(args->items[2]);
            int sockfd = socket(domain, type, protocol);
            return x_int((int64_t)sockfd);
        }
        case 2: { // SYS_BIND
            if (arg_count < 3) return x_int(-1);
            int sockfd = (int)x_as_int(args->items[0]);
            struct sockaddr_in addr;
            memset(&addr, 0, sizeof(addr));
            addr.sin_family = AF_INET;
            addr.sin_addr.s_addr = htonl(INADDR_ANY);
            addr.sin_port = htons(8080); // 固定端口
            int result = bind(sockfd, (struct sockaddr*)&addr, sizeof(addr));
            return x_int((int64_t)result);
        }
        case 3: { // SYS_LISTEN
            if (arg_count < 2) return x_int(-1);
            int sockfd = (int)x_as_int(args->items[0]);
            int backlog = (int)x_as_int(args->items[1]);
            int result = listen(sockfd, backlog);
            return x_int((int64_t)result);
        }
        case 4: { // SYS_ACCEPT
            if (arg_count < 3) return x_int(-1);
            int sockfd = (int)x_as_int(args->items[0]);
            struct sockaddr_in client_addr;
            socklen_t client_len = sizeof(client_addr);
            int clientfd = accept(sockfd, (struct sockaddr*)&client_addr, &client_len);
            return x_int((int64_t)clientfd);
        }
        case 5: { // SYS_RECV
            if (arg_count < 4) return x_int(-1);
            int sockfd = (int)x_as_int(args->items[0]);
            // 这里需要处理缓冲区，暂时返回模拟数据
            return x_int(100); // 模拟读取了100字节
        }
        case 6: { // SYS_SEND
            if (arg_count < 4) return x_int(-1);
            int sockfd = (int)x_as_int(args->items[0]);
            // 这里需要处理缓冲区，暂时返回模拟数据
            return x_int(100); // 模拟发送了100字节
        }
        case 7: { // SYS_CLOSE
            if (arg_count < 1) return x_int(-1);
            int sockfd = (int)x_as_int(args->items[0]);
            int result = close(sockfd);
            return x_int((int64_t)result);
        }
        case 8: { // SYS_SLEEP
            if (arg_count < 1) return x_int(-1);
            int milliseconds = (int)x_as_int(args->items[0]);
            Sleep(milliseconds);
            return x_int(0);
        }
        case 9: { // SYS_CREATE_THREAD
            if (arg_count < 1) return x_int(-1);
            HANDLE thread = CreateThread(NULL, 0, x_thread_start, NULL, 0, NULL);
            if (thread) {
                return x_int((int64_t)thread);
            } else {
                return x_int(-1);
            }
        }
        case 100: { // SYS_DB_CONNECT
            if (arg_count < 1) return x_int(-1);
            // 模拟数据库连接
            return x_int(1); // 模拟连接ID
        }
        case 101: { // SYS_DB_DISCONNECT
            if (arg_count < 1) return x_int(-1);
            // 模拟数据库断开连接
            return x_int(0);
        }
        case 102: { // SYS_DB_QUERY
            if (arg_count < 2) return x_int(-1);
            // 模拟数据库查询
            return x_int(1); // 模拟结果ID
        }
        case 103: { // SYS_DB_EXECUTE
            if (arg_count < 2) return x_int(-1);
            // 模拟数据库执行
            return x_int(1); // 模拟受影响的行数
        }
        case 104: { // SYS_DB_BEGIN_TRANSACTION
            if (arg_count < 1) return x_int(-1);
            // 模拟开始事务
            return x_int(0);
        }
        case 105: { // SYS_DB_COMMIT
            if (arg_count < 1) return x_int(-1);
            // 模拟提交事务
            return x_int(0);
        }
        case 106: { // SYS_DB_ROLLBACK
            if (arg_count < 1) return x_int(-1);
            // 模拟回滚事务
            return x_int(0);
        }
        case 107: { // SYS_DB_PREPARE
            if (arg_count < 2) return x_int(-1);
            // 模拟准备预处理语句
            return x_int(1); // 模拟语句ID
        }
        case 108: { // SYS_DB_EXECUTE_PREPARED
            if (arg_count < 1) return x_int(-1);
            // 模拟执行预处理语句
            return x_int(1); // 模拟结果ID
        }
        case 109: { // SYS_DB_CLOSE_STATEMENT
            if (arg_count < 1) return x_int(-1);
            // 模拟关闭预处理语句
            return x_int(0);
        }
        default:
            return x_int(-1);
    }
}

static inline XValue x_uptime(void) {
#ifdef _WIN32
    FILETIME ft_now, ft_boot;
    GetSystemTimeAsFileTime(&ft_now);
    if (GetSystemTimes(&ft_boot, NULL, NULL)) {
        ULONGLONG now = ((ULONGLONG)ft_now.dwHighDateTime << 32) | ft_now.dwLowDateTime;
        ULONGLONG boot = ((ULONGLONG)ft_boot.dwHighDateTime << 32) | ft_boot.dwLowDateTime;
        double seconds = (now - boot) / 10000000.0;
        return x_float(seconds);
    } else {
        return x_float(0.0);
    }
#else
    struct sysinfo info;
    if (sysinfo(&info) == 0) {
        return x_float((double)info.uptime);
    } else {
        return x_float(0.0);
    }
#endif
}

static inline XValue x_random(XValue max_v) {
    int max = (int)x_as_int(max_v);
    return x_int((int64_t)(rand() % max));
}

static inline XValue x_random_float(void) {
    return x_float((double)rand() / RAND_MAX);
}

static inline void x_srand(XValue seed_v) {
    unsigned int seed = (unsigned int)x_as_int(seed_v);
    srand(seed);
}

static inline XValue x_getuid(void) {
#ifdef _WIN32
    return x_int(0); // Windows 下模拟
#else
    return x_int((int64_t)getuid());
#endif
}

static inline XValue x_getgid(void) {
#ifdef _WIN32
    return x_int(0); // Windows 下模拟
#else
    return x_int((int64_t)getgid());
#endif
}

static inline XValue x_get_username(void) {
#ifdef _WIN32
    char buffer[1024];
    DWORD size = sizeof(buffer);
    if (GetUserName(buffer, &size)) {
        return x_string(buffer);
    } else {
        return x_string("Unknown");
    }
#else
    struct passwd *pwd = getpwuid(getuid());
    if (pwd) {
        return x_string(pwd->pw_name);
    } else {
        return x_string("Unknown");
    }
#endif
}

static inline XValue x_get_groupname(void) {
#ifdef _WIN32
    return x_string("Users"); // Windows 下模拟
#else
    struct group *grp = getgrgid(getgid());
    if (grp) {
        return x_string(grp->gr_name);
    } else {
        return x_string("Unknown");
    }
#endif
}

// Option类型相关函数
static inline XValue x_is_some(XValue v) {
    return x_bool(v.tag != X_NONE && v.tag != X_NULL);
}

static inline XValue x_unwrap(XValue v) {
    // 简单实现，直接返回值本身
    return v;
}

// 集合相关函数
#include <stdarg.h>

static inline XValue x_list(void) {
    return x_array_val(x_array_new(0));
}

static inline XValue x_list_1(XValue v1) {
    XArray *arr = x_array_new(1);
    x_array_push(arr, v1);
    return x_array_val(arr);
}

static inline XValue x_list_2(XValue v1, XValue v2) {
    XArray *arr = x_array_new(2);
    x_array_push(arr, v1);
    x_array_push(arr, v2);
    return x_array_val(arr);
}

static inline XValue x_list_3(XValue v1, XValue v2, XValue v3) {
    XArray *arr = x_array_new(3);
    x_array_push(arr, v1);
    x_array_push(arr, v2);
    x_array_push(arr, v3);
    return x_array_val(arr);
}

static inline XValue x_map(void) {
    return x_map_val(x_map_new());
}

// 数据库相关函数
static inline XValue x_get_connection(XValue pool) {
    // 简单实现，返回一个模拟的连接
    XValue conn = x_map_val(x_map_new());
    x_map_set(conn.as.map, "connection", x_int(1));
    x_map_set(conn.as.map, "connected", x_bool(true));
    x_map_set(conn.as.map, "in_transaction", x_bool(false));
    return conn;
}

static inline XValue x_execute(XValue conn, XValue sql, XValue params) {
    // 简单实现，返回成功
    return x_int(1);
}

static inline XValue x_query(XValue conn, XValue sql, XValue params) {
    // 简单实现，返回成功
    return x_int(1);
}

static inline void x_return_connection(XValue pool, XValue conn) {
    // 简单实现，什么都不做
}

static inline XValue x_begin_transaction(XValue conn) {
    // 简单实现，返回成功
    return x_bool(true);
}

static inline XValue x_commit(XValue conn) {
    // 简单实现，返回成功
    return x_bool(true);
}

// 其他辅助函数
static inline XValue x_to_number(XValue v) {
    // 简单实现，转换为数字
    return x_int(x_as_int(v));
}

static inline XValue x_render_template(XValue engine, XValue name, XValue context) {
    // 简单实现，返回HTML内容
    return x_string("<html><body><h1>Fortune</h1><ul><li>Hello, World!</li><li>Welcome to X Language</li></ul></body></html>");
}

static inline XValue x_build_response(XValue status, XValue body, XValue headers) {
    // 简单实现，打印响应信息
    printf("Response: %lld\n", (long long)x_as_int(status));
    return x_null();
}

// 命令行参数相关函数
static inline XValue x_args(void) {
    // 简单实现，实际需要从main函数传递
    XArray *arr = x_array_new(1);
    x_array_push(arr, x_string("program"));
    return x_array_val(arr);
}

// 路由相关函数
static inline XValue x_router(void) {
    // 简单实现，返回一个模拟的路由对象
    XValue router = x_map_val(x_map_new());
    x_map_set(router.as.map, "routes", x_array_val(x_array_new(0)));
    return router;
}

// 函数指针类型
typedef XValue (*XHandler)(XValue);

static inline void x_get(XValue router, XValue path, XHandler handler) {
    // 简单实现，添加GET路由
    if (router.tag != X_MAP) return;
    XValue routes = x_map_get(router.as.map, "routes");
    if (routes.tag != X_ARRAY) return;
    XValue route = x_map_val(x_map_new());
    x_map_set(route.as.map, "method", x_string("GET"));
    x_map_set(route.as.map, "path", path);
    // 存储函数指针作为整数
    x_map_set(route.as.map, "handler", x_int((int64_t)handler));
    x_array_push(routes.as.arr, route);
}

// 中间件相关函数
static inline XValue x_middleware_chain(void) {
    // 简单实现，返回一个模拟的中间件链
    XValue chain = x_map_val(x_map_new());
    x_map_set(chain.as.map, "middlewares", x_array_val(x_array_new(0)));
    return chain;
}

// 服务器相关函数
static inline XValue x_server(XValue config) {
    // 简单实现，返回一个模拟的服务器对象
    XValue server = x_map_val(x_map_new());
    x_map_set(server.as.map, "config", config);
    x_map_set(server.as.map, "running", x_bool(false));
    x_map_set(server.as.map, "server_fd", x_int(-1));
    return server;
}

static inline void x_start(XValue server) {
    // 简单实现，启动服务器
    if (server.tag != X_MAP) return;
    
    // 标记服务器为运行状态
    x_map_set(server.as.map, "running", x_bool(true));
    
    // 获取服务器配置
    XValue config = x_map_get(server.as.map, "config");
    if (config.tag != X_MAP) return;
    
    XValue host = x_map_get(config.as.map, "host");
    XValue port = x_map_get(config.as.map, "port");
    
    printf("Server starting on %s:%lld...\n", 
           x_as_str(host), (long long)x_as_int(port));
    
    // 初始化Winsock
    WSADATA wsaData;
    int wsaResult = WSAStartup(MAKEWORD(2, 2), &wsaData);
    if (wsaResult != 0) {
        printf("WSAStartup failed: %d\n", wsaResult);
        return;
    }
    
    // 创建服务器socket
    int server_fd = socket(AF_INET, SOCK_STREAM, 0);
    if (server_fd == INVALID_SOCKET) {
        printf("socket failed: %d\n", WSAGetLastError());
        WSACleanup();
        return;
    }
    
    // 设置socket选项
    int opt = 1;
    if (setsockopt(server_fd, SOL_SOCKET, SO_REUSEADDR, (char*)&opt, sizeof(opt)) == SOCKET_ERROR) {
        printf("setsockopt failed: %d\n", WSAGetLastError());
        closesocket(server_fd);
        WSACleanup();
        return;
    }
    
    // 绑定地址和端口
    struct sockaddr_in addr;
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons((u_short)x_as_int(port));
    
    if (bind(server_fd, (struct sockaddr*)&addr, sizeof(addr)) == SOCKET_ERROR) {
        printf("bind failed: %d\n", WSAGetLastError());
        closesocket(server_fd);
        WSACleanup();
        return;
    }
    
    // 开始监听
    if (listen(server_fd, SOMAXCONN) == SOCKET_ERROR) {
        printf("listen failed: %d\n", WSAGetLastError());
        closesocket(server_fd);
        WSACleanup();
        return;
    }
    
    // 更新服务器状态
    x_map_set(server.as.map, "server_fd", x_int((int64_t)server_fd));
    printf("Server started on %s:%lld\n", 
           x_as_str(host), (long long)x_as_int(port));
    
    // 主循环
    while (x_as_bool(x_map_get(server.as.map, "running"))) {
        // 接受连接
        struct sockaddr_in client_addr;
        int client_addr_len = sizeof(client_addr);
        SOCKET client_fd = accept(server_fd, (struct sockaddr*)&client_addr, &client_addr_len);
        if (client_fd == INVALID_SOCKET) {
            printf("accept failed: %d\n", WSAGetLastError());
            continue;
        }
        
        // 处理连接
        char buffer[1024];
        int bytes_received = recv(client_fd, buffer, sizeof(buffer), 0);
        if (bytes_received > 0) {
            // 简单处理HTTP请求
            buffer[bytes_received] = '\0';
            printf("Received request:\n%s\n", buffer);
            
            // 模拟响应
            const char* response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello, World!";
            send(client_fd, response, strlen(response), 0);
        }
        
        // 关闭连接
        closesocket(client_fd);
    }
    
    // 关闭服务器
    closesocket(server_fd);
    WSACleanup();
    printf("Server stopped\n");
}

static inline void x_runtime_init(void) {
    // 简单实现，初始化运行时
    printf("Runtime initialized\n");
}

static inline void x_runtime_cleanup(void) {
    // 简单实现，清理运行时
    printf("Runtime cleanup\n");
}

#endif /* X_RUNTIME_H */
