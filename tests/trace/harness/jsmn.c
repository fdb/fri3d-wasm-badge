#include "jsmn.h"

static jsmntok_t* jsmn_alloc_token(jsmn_parser* parser, jsmntok_t* tokens, size_t num_tokens) {
    if (parser->toknext >= num_tokens) {
        return NULL;
    }
    jsmntok_t* tok = &tokens[parser->toknext++];
    tok->start = -1;
    tok->end = -1;
    tok->size = 0;
    tok->parent = -1;
    tok->type = JSMN_UNDEFINED;
    return tok;
}

static void jsmn_fill_token(jsmntok_t* token, jsmntype_t type, int start, int end) {
    token->type = type;
    token->start = start;
    token->end = end;
    token->size = 0;
}

static int jsmn_parse_primitive(jsmn_parser* parser, const char* js, size_t len,
                                jsmntok_t* tokens, size_t num_tokens) {
    int start = (int)parser->pos;

    for (; parser->pos < len; parser->pos++) {
        char c = js[parser->pos];
        if (c == '\t' || c == '\r' || c == '\n' || c == ' ' ||
            c == ',' || c == ']' || c == '}') {
            jsmntok_t* tok = jsmn_alloc_token(parser, tokens, num_tokens);
            if (!tok) {
                parser->pos = (unsigned int)start;
                return -1;
            }
            jsmn_fill_token(tok, JSMN_PRIMITIVE, start, (int)parser->pos);
            tok->parent = parser->toksuper;
            parser->pos--;
            return 0;
        }
        if (c < 32 || c == '"' || c == ':' || c == '\\') {
            parser->pos = (unsigned int)start;
            return -1;
        }
    }

    jsmntok_t* tok = jsmn_alloc_token(parser, tokens, num_tokens);
    if (!tok) {
        parser->pos = (unsigned int)start;
        return -1;
    }
    jsmn_fill_token(tok, JSMN_PRIMITIVE, start, (int)parser->pos);
    tok->parent = parser->toksuper;
    parser->pos--;
    return 0;
}

static int jsmn_parse_string(jsmn_parser* parser, const char* js, size_t len,
                             jsmntok_t* tokens, size_t num_tokens) {
    int start = (int)parser->pos;
    parser->pos++;

    for (; parser->pos < len; parser->pos++) {
        char c = js[parser->pos];
        if (c == '"') {
            jsmntok_t* tok = jsmn_alloc_token(parser, tokens, num_tokens);
            if (!tok) {
                parser->pos = (unsigned int)start;
                return -1;
            }
            jsmn_fill_token(tok, JSMN_STRING, start + 1, (int)parser->pos);
            tok->parent = parser->toksuper;
            return 0;
        }

        if (c == '\\') {
            parser->pos++;
            if (parser->pos >= len) {
                parser->pos = (unsigned int)start;
                return -1;
            }
            char esc = js[parser->pos];
            switch (esc) {
                case '"':
                case '/':
                case '\\':
                case 'b':
                case 'f':
                case 'r':
                case 'n':
                case 't':
                    break;
                case 'u':
                    for (int i = 0; i < 4; i++) {
                        parser->pos++;
                        if (parser->pos >= len) {
                            parser->pos = (unsigned int)start;
                            return -1;
                        }
                        char h = js[parser->pos];
                        if (!((h >= '0' && h <= '9') ||
                              (h >= 'A' && h <= 'F') ||
                              (h >= 'a' && h <= 'f'))) {
                            parser->pos = (unsigned int)start;
                            return -1;
                        }
                    }
                    break;
                default:
                    parser->pos = (unsigned int)start;
                    return -1;
            }
        }
    }

    parser->pos = (unsigned int)start;
    return -1;
}

void jsmn_init(jsmn_parser* parser) {
    parser->pos = 0;
    parser->toknext = 0;
    parser->toksuper = -1;
}

int jsmn_parse(jsmn_parser* parser, const char* js, size_t len,
               jsmntok_t* tokens, unsigned int num_tokens) {
    for (; parser->pos < len; parser->pos++) {
        char c = js[parser->pos];
        jsmntok_t* tok;

        switch (c) {
            case '{':
            case '[':
                tok = jsmn_alloc_token(parser, tokens, num_tokens);
                if (!tok) {
                    return -1;
                }
                tok->type = (c == '{') ? JSMN_OBJECT : JSMN_ARRAY;
                tok->start = (int)parser->pos;
                tok->parent = parser->toksuper;
                if (parser->toksuper != -1) {
                    tokens[parser->toksuper].size++;
                }
                parser->toksuper = (int)parser->toknext - 1;
                break;
            case '}':
            case ']':
                for (int i = (int)parser->toknext - 1; i >= 0; i--) {
                    tok = &tokens[i];
                    if (tok->start != -1 && tok->end == -1) {
                        if ((tok->type == JSMN_OBJECT && c == '}') ||
                            (tok->type == JSMN_ARRAY && c == ']')) {
                            tok->end = (int)parser->pos + 1;
                            parser->toksuper = tok->parent;
                            break;
                        } else {
                            return -1;
                        }
                    }
                }
                break;
            case '"':
                if (jsmn_parse_string(parser, js, len, tokens, num_tokens) < 0) {
                    return -1;
                }
                if (parser->toksuper != -1) {
                    tokens[parser->toksuper].size++;
                }
                break;
            case '\t':
            case '\r':
            case '\n':
            case ' ':
                break;
            case ':':
                parser->toksuper = (int)parser->toknext - 1;
                break;
            case ',':
                if (parser->toksuper != -1 &&
                    tokens[parser->toksuper].type != JSMN_ARRAY &&
                    tokens[parser->toksuper].type != JSMN_OBJECT) {
                    parser->toksuper = tokens[parser->toksuper].parent;
                }
                break;
            default:
                if (jsmn_parse_primitive(parser, js, len, tokens, num_tokens) < 0) {
                    return -1;
                }
                if (parser->toksuper != -1) {
                    tokens[parser->toksuper].size++;
                }
                break;
        }
    }

    for (unsigned int i = parser->toknext; i > 0; i--) {
        jsmntok_t* tok = &tokens[i - 1];
        if (tok->start != -1 && tok->end == -1) {
            return -1;
        }
    }
    return (int)parser->toknext;
}
