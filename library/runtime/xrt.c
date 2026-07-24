/* xrt.c — X language native runtime implementation.
 *
 * See xrt.h for the public surface. Formatting mirrors the tree-walking
 * interpreter's `format_value` so native output is byte-for-byte identical:
 *   - integers:        12
 *   - floats:          21.0  (integer-valued -> one decimal; else shortest)
 *   - booleans:        true / false
 *   - strings/chars:   printed verbatim (no surrounding quotes)
 *   - lists:           [a, b, c]
 *   - maps:            {k: v, k2: v2}
 */
#include "xrt.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct XList {
    XValue **items;
    long long len;
    long long cap;
};

struct XMapEntry {
    XValue *key;
    XValue *val;
};

struct XMap {
    struct XMapEntry *entries;
    long long len;
    long long cap;
};

struct XValue {
    XTag tag;
    union {
        long long i;
        double d;
        char *s;
        void *p;
        struct XList *list;
        struct XMap *map;
    } u;
};

static void *x_xalloc(size_t n) {
    void *p = malloc(n);
    if (!p) {
        fprintf(stderr, "xrt: out of memory\n");
        abort();
    }
    return p;
}

static XValue *x_alloc(XTag tag) {
    XValue *v = (XValue *)x_xalloc(sizeof(XValue));
    v->tag = tag;
    return v;
}

XValue *x_from_int(long long v) {
    XValue *x = x_alloc(X_INT);
    x->u.i = v;
    return x;
}

XValue *x_from_double(double v) {
    XValue *x = x_alloc(X_DOUBLE);
    x->u.d = v;
    return x;
}

XValue *x_from_bool(long long v) {
    XValue *x = x_alloc(X_BOOL);
    x->u.i = v ? 1 : 0;
    return x;
}

XValue *x_from_char(long long v) {
    XValue *x = x_alloc(X_CHAR);
    x->u.i = v;
    return x;
}

XValue *x_from_str(const char *s) {
    XValue *x = x_alloc(X_STR);
    x->u.s = (char *)(s ? s : "");
    return x;
}

XValue *x_from_ptr(void *p) {
    XValue *x = x_alloc(X_PTR);
    x->u.p = p;
    return x;
}

/* ----------------------------------------------------------------- lists */

XValue *x_list_new(void) {
    XValue *x = x_alloc(X_LIST);
    struct XList *l = (struct XList *)x_xalloc(sizeof(struct XList));
    l->len = 0;
    l->cap = 0;
    l->items = NULL;
    x->u.list = l;
    return x;
}

void x_list_push(XValue *list, XValue *item) {
    if (!list || list->tag != X_LIST)
        return;
    struct XList *l = list->u.list;
    if (l->len == l->cap) {
        l->cap = l->cap ? l->cap * 2 : 4;
        l->items = (XValue **)realloc(l->items, (size_t)l->cap * sizeof(XValue *));
        if (!l->items) {
            fprintf(stderr, "xrt: out of memory\n");
            abort();
        }
    }
    l->items[l->len++] = item;
}

XValue *x_list_get(XValue *list, long long index) {
    if (!list || list->tag != X_LIST)
        return x_from_int(0);
    struct XList *l = list->u.list;
    if (index < 0 || index >= l->len)
        return x_from_int(0);
    return l->items[index];
}

void x_list_set(XValue *list, long long index, XValue *item) {
    if (!list || list->tag != X_LIST)
        return;
    struct XList *l = list->u.list;
    if (index < 0 || index >= l->len)
        return;
    l->items[index] = item;
}

long long x_list_len(XValue *list) {
    if (!list || list->tag != X_LIST)
        return 0;
    return list->u.list->len;
}

XValue *__index__(XValue *collection, long long index) {
    if (!collection)
        return x_from_int(0);
    if (collection->tag == X_LIST)
        return x_list_get(collection, index);
    if (collection->tag == X_STR) {
        const char *s = collection->u.s ? collection->u.s : "";
        size_t len = strlen(s);
        if (index < 0 || (size_t)index >= len)
            return x_from_str("");
        return x_from_char((unsigned char)s[index]);
    }
    return x_from_int(0);
}

/* ------------------------------------------------------------------ maps */

XValue *x_map_new(void) {
    XValue *x = x_alloc(X_MAP);
    struct XMap *m = (struct XMap *)x_xalloc(sizeof(struct XMap));
    m->len = 0;
    m->cap = 0;
    m->entries = NULL;
    x->u.map = m;
    return x;
}

static int x_value_eq(XValue *a, XValue *b) {
    if (a == b)
        return 1;
    if (!a || !b || a->tag != b->tag)
        return 0;
    switch (a->tag) {
    case X_INT:
    case X_BOOL:
    case X_CHAR:
        return a->u.i == b->u.i;
    case X_DOUBLE:
        return a->u.d == b->u.d;
    case X_STR: {
        const char *sa = a->u.s ? a->u.s : "";
        const char *sb = b->u.s ? b->u.s : "";
        return strcmp(sa, sb) == 0;
    }
    case X_PTR:
        return a->u.p == b->u.p;
    default:
        return 0;
    }
}

void x_map_put(XValue *map, XValue *key, XValue *value) {
    if (!map || map->tag != X_MAP)
        return;
    struct XMap *m = map->u.map;
    for (long long i = 0; i < m->len; i++) {
        if (x_value_eq(m->entries[i].key, key)) {
            m->entries[i].val = value;
            return;
        }
    }
    if (m->len == m->cap) {
        m->cap = m->cap ? m->cap * 2 : 4;
        m->entries = (struct XMapEntry *)realloc(
            m->entries, (size_t)m->cap * sizeof(struct XMapEntry));
        if (!m->entries) {
            fprintf(stderr, "xrt: out of memory\n");
            abort();
        }
    }
    m->entries[m->len].key = key;
    m->entries[m->len].val = value;
    m->len++;
}

XValue *x_map_get(XValue *map, XValue *key) {
    if (!map || map->tag != X_MAP)
        return x_from_int(0);
    struct XMap *m = map->u.map;
    for (long long i = 0; i < m->len; i++) {
        if (x_value_eq(m->entries[i].key, key))
            return m->entries[i].val;
    }
    return x_from_int(0);
}

/* ------------------------------------------------------------- unboxing */

long long x_as_int(XValue *v) {
    if (!v)
        return 0;
    switch (v->tag) {
    case X_INT:
    case X_BOOL:
    case X_CHAR:
        return v->u.i;
    case X_DOUBLE:
        return (long long)v->u.d;
    default:
        return 0;
    }
}

double x_as_double(XValue *v) {
    if (!v)
        return 0.0;
    if (v->tag == X_DOUBLE)
        return v->u.d;
    if (v->tag == X_INT)
        return (double)v->u.i;
    return 0.0;
}

long long x_as_bool(XValue *v) {
    if (!v)
        return 0;
    return v->u.i ? 1 : 0;
}

char *x_as_str(XValue *v) {
    if (!v)
        return (char *)"";
    if (v->tag == X_STR)
        return v->u.s ? v->u.s : (char *)"";
    return x_fmt_value(v);
}

void *x_as_ptr(XValue *v) {
    if (!v)
        return NULL;
    if (v->tag == X_PTR)
        return v->u.p;
    return (void *)v;
}

/* ------------------------------------------------------------ formatting */

/* dynamic string buffer */
struct XBuf {
    char *data;
    size_t len;
    size_t cap;
};

static void buf_init(struct XBuf *b) {
    b->cap = 32;
    b->len = 0;
    b->data = (char *)x_xalloc(b->cap);
    b->data[0] = '\0';
}

static void buf_reserve(struct XBuf *b, size_t extra) {
    if (b->len + extra + 1 > b->cap) {
        while (b->len + extra + 1 > b->cap)
            b->cap *= 2;
        b->data = (char *)realloc(b->data, b->cap);
        if (!b->data) {
            fprintf(stderr, "xrt: out of memory\n");
            abort();
        }
    }
}

static void buf_puts(struct XBuf *b, const char *s) {
    size_t n = strlen(s);
    buf_reserve(b, n);
    memcpy(b->data + b->len, s, n);
    b->len += n;
    b->data[b->len] = '\0';
}

/* Format a double the way the interpreter does. */
static void fmt_double(struct XBuf *b, double d) {
    char tmp[64];
    double t = d < 0 ? -d : d;
    /* integer-valued (and finite) -> one decimal place */
    if (t == (double)(long long)t && t < 1e18) {
        snprintf(tmp, sizeof(tmp), "%.1f", d);
        buf_puts(b, tmp);
        return;
    }
    /* otherwise: shortest representation that round-trips */
    for (int prec = 1; prec <= 17; prec++) {
        snprintf(tmp, sizeof(tmp), "%.*g", prec, d);
        if (strtod(tmp, NULL) == d)
            break;
    }
    buf_puts(b, tmp);
}

static void fmt_into(struct XBuf *b, XValue *v) {
    if (!v) {
        buf_puts(b, "null");
        return;
    }
    char tmp[64];
    switch (v->tag) {
    case X_INT:
        snprintf(tmp, sizeof(tmp), "%lld", v->u.i);
        buf_puts(b, tmp);
        break;
    case X_DOUBLE:
        fmt_double(b, v->u.d);
        break;
    case X_BOOL:
        buf_puts(b, v->u.i ? "true" : "false");
        break;
    case X_CHAR: {
        /* encode codepoint as UTF-8 */
        unsigned long c = (unsigned long)v->u.i;
        char enc[5];
        int n = 0;
        if (c < 0x80) {
            enc[n++] = (char)c;
        } else if (c < 0x800) {
            enc[n++] = (char)(0xC0 | (c >> 6));
            enc[n++] = (char)(0x80 | (c & 0x3F));
        } else if (c < 0x10000) {
            enc[n++] = (char)(0xE0 | (c >> 12));
            enc[n++] = (char)(0x80 | ((c >> 6) & 0x3F));
            enc[n++] = (char)(0x80 | (c & 0x3F));
        } else {
            enc[n++] = (char)(0xF0 | (c >> 18));
            enc[n++] = (char)(0x80 | ((c >> 12) & 0x3F));
            enc[n++] = (char)(0x80 | ((c >> 6) & 0x3F));
            enc[n++] = (char)(0x80 | (c & 0x3F));
        }
        enc[n] = '\0';
        buf_puts(b, enc);
        break;
    }
    case X_STR:
        buf_puts(b, v->u.s ? v->u.s : "");
        break;
    case X_PTR:
        snprintf(tmp, sizeof(tmp), "Pointer(0x%llx)",
                 (unsigned long long)(size_t)v->u.p);
        buf_puts(b, tmp);
        break;
    case X_LIST: {
        struct XList *l = v->u.list;
        buf_puts(b, "[");
        for (long long i = 0; i < l->len; i++) {
            if (i)
                buf_puts(b, ", ");
            fmt_into(b, l->items[i]);
        }
        buf_puts(b, "]");
        break;
    }
    case X_MAP: {
        struct XMap *m = v->u.map;
        buf_puts(b, "{");
        for (long long i = 0; i < m->len; i++) {
            if (i)
                buf_puts(b, ", ");
            fmt_into(b, m->entries[i].key);
            buf_puts(b, ": ");
            fmt_into(b, m->entries[i].val);
        }
        buf_puts(b, "}");
        break;
    }
    case X_UNIT:
        buf_puts(b, "()");
        break;
    default:
        buf_puts(b, "");
        break;
    }
}

char *x_fmt_value(XValue *v) {
    struct XBuf b;
    buf_init(&b);
    fmt_into(&b, v);
    return b.data;
}

char *x_to_str(XValue *v) { return x_fmt_value(v); }

char *x_str_concat(const char *a, const char *b) {
    if (!a)
        a = "";
    if (!b)
        b = "";
    size_t la = strlen(a), lb = strlen(b);
    char *out = (char *)x_xalloc(la + lb + 1);
    memcpy(out, a, la);
    memcpy(out + la, b, lb);
    out[la + lb] = '\0';
    return out;
}

/* ------------------------------------------------------------- printing */

void x_print(XValue *v) {
    char *s = x_fmt_value(v);
    fputs(s, stdout);
    fputc('\n', stdout);
    free(s);
}

void x_print_inline(XValue *v) {
    char *s = x_fmt_value(v);
    fputs(s, stdout);
    free(s);
}

void x_print_newline(void) { fputc('\n', stdout); }

/* ------------------------------------------- pi digits (Rabinowitz–Wagon) */

char *compute_pi_digits(long long n) {
    if (n <= 0) {
        char *empty = (char *)x_xalloc(1);
        empty[0] = '\0';
        return empty;
    }
    /* Generate n+1 digits then drop the leading 0 that precedes '3'. */
    long long want = n + 1;
    long long len = (10 * want) / 3 + 2;
    int *a = (int *)x_xalloc((size_t)len * sizeof(int));
    for (long long i = 0; i < len; i++)
        a[i] = 2;

    char *buf = (char *)x_xalloc((size_t)want + 2);
    long long produced = 0;
    int nines = 0;
    int predigit = 0;

    while (produced < want) {
        long long q = 0;
        for (long long i = len - 1; i >= 0; i--) {
            long long x = 10 * a[i] + q * (i + 1);
            a[i] = (int)(x % (2 * i + 1));
            q = x / (2 * i + 1);
        }
        a[0] = (int)(q % 10);
        q = q / 10;
        if (q == 9) {
            nines++;
        } else if (q == 10) {
            buf[produced++] = (char)('0' + predigit + 1);
            for (int k = 0; k < nines && produced < want; k++)
                buf[produced++] = '0';
            predigit = 0;
            nines = 0;
        } else {
            buf[produced++] = (char)('0' + predigit);
            predigit = (int)q;
            for (int k = 0; k < nines && produced < want; k++)
                buf[produced++] = '9';
            nines = 0;
        }
    }
    free(a);

    char *out = (char *)x_xalloc((size_t)n + 1);
    /* Skip leading 0; take next n digits (starting with 3). */
    memcpy(out, buf + 1, (size_t)n);
    out[n] = '\0';
    free(buf);
    return out;
}

/* ----------------------------------------------------------- simple regex */

static size_t x_pattern_len(const char *pat) {
    size_t len = 0;
    for (const char *p = pat; *p;) {
        if (*p == '[') {
            while (*p && *p != ']')
                p++;
            if (*p == ']')
                p++;
            len++;
        } else {
            p++;
            len++;
        }
    }
    return len;
}

static int x_match_at(const char *text, size_t start, const char *pat) {
    size_t ti = start;
    const char *p = pat;
    size_t tlen = strlen(text);
    while (*p) {
        if (ti >= tlen)
            return 0;
        if (*p == '[') {
            p++;
            int matched = 0;
            while (*p && *p != ']') {
                if (text[ti] == *p)
                    matched = 1;
                p++;
            }
            if (!matched)
                return 0;
            if (*p == ']')
                p++;
            ti++;
        } else {
            if (text[ti] != *p)
                return 0;
            ti++;
            p++;
        }
    }
    return 1;
}

static long long x_regex_count_one(const char *text, const char *pattern) {
    if (strchr(pattern, '|')) {
        char *dup = strdup(pattern);
        if (!dup)
            return 0;
        long long total = 0;
        char *save = NULL;
        for (char *tok = strtok_r(dup, "|", &save); tok;
             tok = strtok_r(NULL, "|", &save)) {
            total += x_regex_count_one(text, tok);
        }
        free(dup);
        return total;
    }
    if (strchr(pattern, '[')) {
        size_t tlen = strlen(text);
        size_t plen = x_pattern_len(pattern);
        if (plen == 0)
            return 0;
        long long count = 0;
        size_t i = 0;
        while (i + plen <= tlen) {
            if (x_match_at(text, i, pattern)) {
                count++;
                i += plen;
            } else {
                i++;
            }
        }
        return count;
    }
    long long count = 0;
    size_t plen = strlen(pattern);
    if (plen == 0)
        return 0;
    const char *s = text;
    while ((s = strstr(s, pattern)) != NULL) {
        count++;
        s += plen;
    }
    return count;
}

long long regex_match_count(const char *text, const char *pattern) {
    if (!text)
        text = "";
    if (!pattern)
        pattern = "";
    return x_regex_count_one(text, pattern);
}

char *regex_replace_all(const char *text, const char *pattern,
                        const char *replacement) {
    if (!text)
        text = "";
    if (!pattern)
        pattern = "";
    if (!replacement)
        replacement = "";

    size_t tlen = strlen(text);
    struct XBuf b;
    buf_init(&b);

    if (strchr(pattern, '[') || strchr(pattern, '|')) {
        char *dup = strdup(pattern);
        char *alts[64];
        int nalt = 0;
        if (dup) {
            char *save = NULL;
            for (char *tok = strtok_r(dup, "|", &save); tok && nalt < 64;
                 tok = strtok_r(NULL, "|", &save)) {
                alts[nalt++] = tok;
            }
        }
        size_t i = 0;
        while (i < tlen) {
            int matched = 0;
            for (int a = 0; a < nalt; a++) {
                size_t plen = x_pattern_len(alts[a]);
                if (i + plen <= tlen && x_match_at(text, i, alts[a])) {
                    buf_puts(&b, replacement);
                    i += plen;
                    matched = 1;
                    break;
                }
            }
            if (!matched) {
                char ch[2] = {text[i], '\0'};
                buf_puts(&b, ch);
                i++;
            }
        }
        free(dup);
    } else {
        size_t plen = strlen(pattern);
        size_t i = 0;
        if (plen == 0) {
            buf_puts(&b, text);
        } else {
            while (i < tlen) {
                if (i + plen <= tlen && memcmp(text + i, pattern, plen) == 0) {
                    buf_puts(&b, replacement);
                    i += plen;
                } else {
                    char ch[2] = {text[i], '\0'};
                    buf_puts(&b, ch);
                    i++;
                }
            }
        }
    }
    return b.data;
}
