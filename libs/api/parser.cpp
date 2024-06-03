#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <stdint.h>
#include <pugixml.hpp>
#include <iostream>
#include <sstream>
#include <string>
#include <string.h>

#include "includes/parser.hpp"

#define IF_EQUALS(s1, s2) (strcmp(s1, s2) == 0)

// RESPONSE

using namespace pugi;

message create_response(message_status status, char* err) {
    message m = {
        .type = RESPONSE,
        .status = status,
        .err = err,
    };

    if(err == NULL) {
        m.err = NULL;
    } else {
        m.err = (char*)malloc((strlen(err)+1)*sizeof(char));
        strcpy(m.err, err);
    }

    return m;
}

void destroy_response(message* m) {
    free(m->err);
}

char* deserialize_response(message* response) {
    if(response->type != RESPONSE) {
        return NULL;
    }

    pugi::xml_document doc;
    pugi::xml_node root = doc.append_child("root");
    root.append_child("type")
        .text()
        .set(MESSAGE_TYPE_NAME[response->type]);
    root.append_child("status")
        .text()
        .set(MESSAGE_STATUS_NAME[response->status]);

    pugi::xml_node err = root.append_child("err");
    if (response->err != NULL) {
        err.text().set(response->err);
    }

    std::stringstream ss;
    doc.save(ss);
    std::string xmlString = ss.str();
    const char* xml_str = xmlString.c_str();
    size_t len = strlen(xml_str);
    char* deserialized_response = (char*)malloc((len+1) * sizeof(char));
    strcpy(deserialized_response, xml_str);
    return deserialized_response;
}

bool serialize_response(message* message, const char* raw_xml) {
    pugi::xml_document doc;
    pugi::xml_parse_result result = doc.load_string(raw_xml);
    if(!result) {
        return false;
    }
    pugi::xml_node root = doc.child("root");
    pugi::xml_node type = root.child(TYPE_NODE);
    pugi::xml_node status = root.child(STATUS_NODE);
    pugi::xml_node err = root.child(ERROR_NODE);

    if(!type || !status || !err ){
        return false;
    }

    const char_t* type_value = type.text().as_string();
    if(IF_EQUALS(type_value, MESSAGE_TYPE_NAME[0])) {
        message->type = RESPONSE;
    } else if(IF_EQUALS(type_value, MESSAGE_TYPE_NAME[1])) {
        message->type = USER_REGISTRATION;
    } else {
        printf("DUPSKO");
        return false;
    }

    const char_t* status_value = status.text().as_string();
    if(IF_EQUALS(status_value, MESSAGE_STATUS_NAME[0])) {
        message->status = OK;
    } else if(IF_EQUALS(status_value, MESSAGE_STATUS_NAME[1])) {
        message->status = ERR;
    } else {
        return false;
    }

    const char_t* err_value = err.text().as_string();
    if(!IF_EQUALS(err_value, "")) {
        message->err = (char*)malloc((strlen(err_value)+1)*sizeof(char));
        strcpy(message->err, err_value);
    }
    return true;
}
