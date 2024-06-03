#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include "../includes/parser.hpp"

int main() {
    message m = create_response(OK, "Basic error message");
    printf("BASIC MESSAGE:\ntype-%i\nstatus-%i\nerr-%s\n", m.type, m.status, m.err);
    destroy_response(&m);
    printf("\n\n");

    message m2;
    bool result = serialize_response(&m2, "\
    <root>\
    <type>response</type>\
    <status>err</status>\
    <error>test error</error>\
    </root>\
    ");
    if(result == true) {
        printf("SERIALIZED MESSAGE:\n");
        printf("PARSED TYPE: %i\n", m2.type);
        printf("PARSED STATUS: %i\n", m2.status);
        printf("ERR: %s\n", m2.err);
    } else {
        printf("Could not serialize message");
    }
    char* res = deserialize_response(&m2);
    if(res != NULL) {
        printf("DESERIALIZED RESPONSE:\n");
        printf("%s\n", res);
    } else {
        printf("Could not deserialize message...\n");
    }
    free(res);
    destroy_response(&m2);

    return 0;
}