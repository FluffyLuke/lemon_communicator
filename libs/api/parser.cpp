#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include <stdint.h>
#include <pugixml.hpp>
#include <sstream>
#include <string>
#include <string.h>

#include "includes/parser.hpp"

#define IF_EQUALS(s1, s2) (strcmp(s1, s2) == 0)

// RESPONSE

using namespace pugi;

void init_message(message_t* m, message_status status, const char* err) {
    m->type = RESPONSE;
    m->status = status;
    m->err = NULL;
    if(err != NULL) {
        m->err = (char*)malloc((strlen(err)+1)*sizeof(char));
        strcpy(m->err, err);
    }
}

void destroy_message(message_t* m) {
    if(m->err != NULL) {
        free(m->err);
    }
    switch (m->type) {
        case LOGIN: {
            free(m->data.login.key);
            free(m->data.login.password);
            break;
        }
        default: {
            break;
        }
    }
}

char* serialize_message(message_t* m) {
    if(m->type != RESPONSE) {
        return NULL;
    }

    pugi::xml_document doc;
    pugi::xml_node root = doc.append_child("root");
    root.append_child("type")
        .text()
        .set(MESSAGE_TYPE_NAME[m->type]);
    root.append_child("status")
        .text()
        .set(MESSAGE_STATUS_NAME[m->status]);

    pugi::xml_node err = root.append_child("err");
    if (m->err != NULL) {
        err.text().set(m->err);
    }

    switch(m->type) {
        case LOGIN: {
            root.append_child("key")
                .text()
                .set(m->data.login.key);
            root.append_child("password")
                .text()
                .set(m->data.login.password);
        }
        default:
            break;
    }

    std::stringstream ss;
    doc.save(ss);
    std::string xmlString = ss.str();
    const char* xml_str = xmlString.c_str();
    size_t len = strlen(xml_str);
    char* deserialized_message = (char*)malloc((len+1) * sizeof(char));
    strcpy(deserialized_message, xml_str);

    return deserialized_message;
}

void deserialize_message(message_t* message, const char* raw_xml) {
    pugi::xml_document doc;
    pugi::xml_parse_result result = doc.load_string(raw_xml);
    printf("1!\n");
    if(!result) {
        message->type = PARSE_ERR;
        return;
    }
    pugi::xml_node root = doc.child("root");
    pugi::xml_node type = root.child(TYPE_NODE);
    pugi::xml_node status = root.child(STATUS_NODE);
    pugi::xml_node err = root.child(ERROR_NODE);

    printf("2!\n");
    if(!type || !status || !err ){
        message->type = PARSE_ERR;
        return;
    }

    printf("Parsing message!\n");
    const char_t* type_value = type.text().as_string();
    if(IF_EQUALS(type_value, MESSAGE_TYPE_NAME[0])) {
        message->type = RESPONSE;
    } else if(IF_EQUALS(type_value, MESSAGE_TYPE_NAME[1])) {
        message->type = LOGIN;
    } else {
        fprintf(stderr, "Cannot parse message!\n");
        message->type = PARSE_ERR;
        return;
    }

    const char_t* status_value = status.text().as_string();
    if(IF_EQUALS(status_value, MESSAGE_STATUS_NAME[0])) {
        message->status = OK;
    } else if(IF_EQUALS(status_value, MESSAGE_STATUS_NAME[1])) {
        message->status = ERR;
    } else {
        fprintf(stderr, "Cannot parse message!\n");
        message->type = PARSE_ERR;
        return;
    }

    const char_t* err_value = err.text().as_string();
    if(!IF_EQUALS(err_value, "")) {
        message->err = (char*)malloc((strlen(err_value)+1)*sizeof(char));
        strcpy(message->err, err_value);
    }

    switch(message->type) {
        case LOGIN: {
            pugi::xml_node key = doc.child("key");
            pugi::xml_node password = doc.child("password");
            if(!key || !password) {
                fprintf(stderr, "Cannot parse login message!");
                message->type = PARSE_ERR;
                return;
            }
            const char_t* key_value = key.text().as_string();
            const char_t* password_value = password.text().as_string();

            message->data.login.key = (char*)malloc((strlen(key_value)+1)*sizeof(char));
            strcpy(message->data.login.key, key_value);
            message->data.login.key = (char*)malloc((strlen(password_value)+1)*sizeof(char));
            strcpy(message->data.login.key, password_value);
            break;
        }
        default: {
            break;
        }
    }
}
