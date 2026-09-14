/*
 * The probe: drives a contract module through the header's own types, the way
 * a host would, and exits non-zero on the first departure. Linked with the
 * module's source, so it exercises the entrypoint directly rather than through
 * a loader; `xmip probe` on the built library covers the loading.
 *
 * One probe for every language technology of the capability — c, cpp, java,
 * python — because they differ only in what the descriptor's standard says
 * (ADR-0044). The build passes that as XMIP_STANDARD, unquoted:
 * -DXMIP_STANDARD=cpp. Compiled as C or as C++, whichever the module is.
 */

#include "xmip_module.h"

#include <stdio.h>
#include <string.h>

#ifndef XMIP_STANDARD
#error "XMIP_STANDARD names the standard the module must report, e.g. -DXMIP_STANDARD=c"
#endif
#define XMIP_STRING_(name) #name
#define XMIP_STRING(name) XMIP_STRING_(name)
#define XMIP_STANDARD_NAME XMIP_STRING(XMIP_STANDARD)

#ifdef __cplusplus
extern "C"
#endif
XmipStatus xmip_create_module_v1(const XmipHost *host, XmipModule *out);

typedef struct { const uint8_t *bytes; size_t len; size_t at; } Source;

static int64_t read_source(void *ctx, uint8_t *buf, size_t len) {
    Source *s = (Source *)ctx;
    size_t left = s->len - s->at;
    size_t n = left < len ? left : len;
    /* Short reads on purpose: a short read is not end of stream. */
    if (n > 3) n = 3;
    memcpy(buf, s->bytes + s->at, n);
    s->at += n;
    return (int64_t)n;
}

static int check(int condition, const char *what) {
    if (!condition) { fprintf(stderr, "FAILED: %s\n", what); return 1; }
    printf("ok  %s\n", what);
    return 0;
}

int main(void) {
    XmipHost host = { XMIP_ABI_VERSION, NULL, NULL, NULL, NULL };
    XmipHost foreign = { 99u, NULL, NULL, NULL, NULL };
    XmipModule module;
    const XmipContractVtable *table;
    void *contract = NULL;
    const XmipDiagnostic *diagnostics = NULL;
    size_t count = 7;
    XmipStr descriptor = { (const uint8_t *)"any", 3 };
    XmipStr key = { (const uint8_t *)"descriptor", 10 };
    XmipStr implied = { NULL, 0 };
    Source source = { (const uint8_t *)"xmip ping-pong", 14, 0 };
    XmipReader reader = { &source, read_source };
    const char *standard = XMIP_STANDARD_NAME;
    size_t standard_len = strlen(standard);
    int failures = 0;

    memset(&module, 0, sizeof module);
    failures += check(xmip_create_module_v1(&foreign, &module) == XMIP_E_UNSUPPORTED,
                      "a foreign abi_version is refused");
    failures += check(module.vtable == NULL, "and *out is left untouched");
    failures += check(xmip_create_module_v1(&host, &module) == XMIP_OK, "the module is created");
    failures += check(module.descriptor.module.len == 8 &&
                      memcmp(module.descriptor.module.ptr, "contract", 8) == 0,
                      "the descriptor names the contract trait");
    failures += check(module.descriptor.standard.len == standard_len &&
                      memcmp(module.descriptor.standard.ptr, standard, standard_len) == 0,
                      "and the standard is " XMIP_STANDARD_NAME);
    table = (const XmipContractVtable *)module.vtable;
    failures += check(table->header.start(module.state) == XMIP_OK, "start");
    failures += check(table->load(module.state, descriptor, &contract) == XMIP_OK && contract,
                      "a descriptor loads into a contract");
    failures += check(table->validate(module.state, contract, &reader, &diagnostics, &count) == XMIP_OK
                      && count == 0, "a stream holds with no diagnostics");
    failures += check(source.at == source.len, "and was read to its end in short reads");
    failures += check(table->implies(module.state, contract, key, &implied) == XMIP_OK &&
                      implied.len == 3 && memcmp(implied.ptr, "any", 3) == 0,
                      "implies answers the bound descriptor");
    key.ptr = (const uint8_t *)"nothing"; key.len = 7;
    failures += check(table->implies(module.state, contract, key, &implied) == XMIP_E_NOT_FOUND,
                      "and NOT_FOUND for what it does not determine");
    table->release(module.state, contract);
    failures += check(table->header.stop(module.state) == XMIP_OK, "stop");
    failures += check(module.last_error(module.state).len == 0, "no error was recorded");
    module.destroy(module.state);
    printf(failures ? "FAILED\n" : "OK\n");
    return failures ? 1 : 0;
}
