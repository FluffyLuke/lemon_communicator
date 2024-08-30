#include <arpa/inet.h>
#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include <netinet/in.h>
#include <stdint.h>
#include <sstream>
#include <string>
#include <string.h>
#include <stdlib.h>
#include <sys/socket.h>

extern "C" {
#include "../../vec/src/vec.h"
}

#include "../includes/parser.hpp"
#include "../pugixml/src/pugixml.hpp"

#define IF_EQUALS(s1, s2) (strcmp(s1, s2) == 0)

// RESPONSE

using namespace pugi;

void init_message(message_t* m, message_type type, message_status status, const char* err) {
    m->type = type;
    m->status = status;
    m->err = NULL;
    if(err != NULL) {
        m->err = (char*)malloc((strlen(err)+1)*sizeof(char));
        strcpy(m->err, err);
    }
}

extern "C" void destroy_message(message_t* m) {
    if(m->err != NULL) {
        free(m->err);
    }
    switch (m->type) {
        case LOGIN: {
            if(m->data.login.key != NULL)
                free(m->data.login.key);
            if(m->data.login.password != NULL)
                free(m->data.login.password);
            break;
        }
        case LOGIN_RETURN: {
            if(m->data.login_r.token != NULL) 
                free(m->data.login_r.token);
            break;
        }
        case NETWORK_STATE:
            if(m->data.network.token != NULL) 
                free(m->data.login_r.token);
            break;
        case NETWORK_STATE_RETURN:
            vec_deinit(&m->data.network_r.clients);
        default: {
            break;
        }
    }
}

extern "C" char* serialize_message(message_t* m) {
    // if(m->type != RESPONSE) {
    //     return NULL;
    // }

    pugi::xml_document doc;
    pugi::xml_node root = doc.append_child(ROOT_NODE);
    root.append_child(TYPE_NODE)
        .text()
        .set(MESSAGE_TYPE_NAME[m->type]);
    root.append_child(STATUS_NODE)
        .text()
        .set(MESSAGE_STATUS_NAME[m->status]);

    pugi::xml_node err = root.append_child(ERROR_NODE);
    if (m->err != NULL) {
        err.text().set(m->err);
    }

    switch(m->type) {
        case LOGIN: {
            root.append_child(KEY_NODE)
                .text()
                .set(m->data.login.key);
            root.append_child(PASSWORD_NODE)
                .text()
                .set(m->data.login.password);
            break;
        }
        case LOGIN_RETURN: {
            if(m->data.login_r.token != NULL) {
                root.append_child(TOKEN_NODE)
                    .text()
                    .set(m->data.login_r.token);
            }
            break;
        }
        case NETWORK_STATE: {
            root.append_child(TOKEN_NODE)
                .text()
                .set(m->data.network.token);
            break;
        }

        case NETWORK_STATE_RETURN: {
            pugi::xml_node clients = root.append_child(CLIENTS_NODE);
            client_body_t cb;
            int32_t i;
            vec_foreach(&m->data.network_r.clients, cb, i) {
                printf("TESTING: %d\n", i);
                char ip[INET_ADDRSTRLEN];
                uint16_t port;

                inet_ntop(AF_INET, &(cb.ip.sin_addr), ip, INET_ADDRSTRLEN);
                port = htons(cb.ip.sin_port);

                pugi::xml_node client = clients.append_child(SINGLE_CLIENT_NODE);
                client.append_child(FIRST_NAME_NODE)
                    .text()
                    .set(cb.first_name);
                client.append_child(LAST_NAME_NODE)
                    .text()
                    .set(cb.last_name);
                client.append_child(EMAIL_NODE)
                    .text()
                    .set(cb.email);
                client.append_child(IPV4_NODE)
                    .text()
                    .set(ip);
                client.append_child(PORT_NODE)
                    .text()
                    .set(port);
            }
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
    if(!result) {
        fprintf(stderr, "Cannot parse message: invalid xml\n");
        message->type = PARSE_ERR;
        return;
    }
    pugi::xml_node root = doc.child("root");
    pugi::xml_node type = root.child(TYPE_NODE);
    pugi::xml_node status = root.child(STATUS_NODE);
    pugi::xml_node err = root.child(ERROR_NODE);

    if(!type || !status || !err ){
        message->type = PARSE_ERR;
        return;
    }

    printf("Parsing message!\n");
    const char_t* type_value = type.text().as_string();

    if(IF_EQUALS(type_value, MESSAGE_TYPE_NAME[0])) {
        message->type = RESPONSE;
    } else if(IF_EQUALS(type_value, MESSAGE_TYPE_NAME[1])) {
        message->type = PARSE_ERR;
    } else if(IF_EQUALS(type_value, MESSAGE_TYPE_NAME[2])) {
        message->type = LOGIN;
    } else if(IF_EQUALS(type_value, MESSAGE_TYPE_NAME[3])) {
        message->type = LOGIN_RETURN;
    } else if(IF_EQUALS(type_value, MESSAGE_TYPE_NAME[4])) {
        message->type = NETWORK_STATE;
    } else if(IF_EQUALS(type_value, MESSAGE_TYPE_NAME[5])) {
        message->type = NETWORK_STATE_RETURN;
    } else {
        fprintf(stderr, "Cannot parse message: wrong type!\n");
        message->type = PARSE_ERR;
        return;
    }

    const char_t* status_value = status.text().as_string();
    if(IF_EQUALS(status_value, MESSAGE_STATUS_NAME[0])) {
        message->status = OK;
    } else if(IF_EQUALS(status_value, MESSAGE_STATUS_NAME[1])) {
        message->status = ERR;
    } else {
        fprintf(stderr, "Cannot parse message: wrong status!\n");
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
            pugi::xml_node key = root.child(KEY_NODE);
            pugi::xml_node password = root.child(PASSWORD_NODE);
            if(!key || !password) {
                fprintf(stderr, "Cannot parse login message!\n");
                message->type = PARSE_ERR;
                return;
            }
            const char_t* key_value = key.text().as_string();
            const char_t* password_value = password.text().as_string();

            message->data.login.key = (char*)malloc((strlen(key_value)+1)*sizeof(char));
            strcpy(message->data.login.key, key_value);
            message->data.login.password = (char*)malloc((strlen(password_value)+1)*sizeof(char));
            strcpy(message->data.login.password, password_value);
            break;
        }
        case LOGIN_RETURN: {
            pugi::xml_node token = root.child(TOKEN_NODE);
            if(!token) {
                fprintf(stderr, "Cannot parse returned login message!\n");
                message->type = PARSE_ERR;
                return;
            }
            const char_t* token_value = token.text().as_string();
            message->data.login_r.token = (char*)malloc(TOKEN_BUFFER_SIZE);
            memset(message->data.login_r.token, 0, TOKEN_BUFFER_SIZE);
            strncpy(message->data.login_r.token, token_value, TOKEN_BUFFER_SIZE);
            break;
        }
        case NETWORK_STATE: {
            pugi::xml_node token = root.child(TOKEN_NODE);
            if(!token) {
                fprintf(stderr, "Cannot parse network state message!\n");
                message->type = PARSE_ERR;
                return;
            }
            const char_t* token_value = token.text().as_string();
            message->data.login_r.token = (char*)malloc(TOKEN_BUFFER_SIZE);
            memset(message->data.login_r.token, 0, TOKEN_BUFFER_SIZE);
            strncpy(message->data.login_r.token, token_value, TOKEN_BUFFER_SIZE);
            break;
        }
        case NETWORK_STATE_RETURN: {
            pugi::xml_node clients = root.child(CLIENTS_NODE);
            if(!clients) {
                fprintf(stderr, "Cannot parse network state return message - no clients node\n");
                message->type = PARSE_ERR;
                return;
            }
            client_body_vec_t clients_vec;
            vec_init(&clients_vec);

            pugi::xml_node client = clients.first_child();
            while(client) {
                pugi::xml_node first_name_node = client.child(FIRST_NAME_NODE);
                pugi::xml_node last_name_node = client.child(LAST_NAME_NODE);
                pugi::xml_node email_node = client.child(EMAIL_NODE);
                pugi::xml_node ipv4_node = client.child(IPV4_NODE);
                pugi::xml_node port_node = client.child(PORT_NODE);

                if(!first_name_node || !last_name_node || !email_node || !ipv4_node || !port_node) {
                    fprintf(stderr, "Cannot parse network state return message - client node is incomplete\n");
                }

                const char_t* first_name_value = first_name_node.text().as_string();
                const char_t* last_name_value = last_name_node.text().as_string();
                const char_t* email_value = email_node.text().as_string();
                const char_t* ipv4_value = ipv4_node.text().as_string();
                const char_t* port_value = port_node.text().as_string();

                uint16_t parsed_port = atoi(port_value);
                if(!parsed_port) {
                    fprintf(stderr, "Cannot parse network state return message - port cannot be parsed\n");
                    message->type = PARSE_ERR;
                    return;
                }

                client_body_t cb;
                memset(cb.first_name, 0, FIRST_NAME_LEN);
                memset(cb.last_name, 0, LAST_NAME_LEN);
                memset(cb.email, 0, EMAIL_LEN);

                // TODO check for length of strings
                strcpy(cb.first_name, first_name_value);
                strcpy(cb.last_name, last_name_value);
                strcpy(cb.email, email_value);

                cb.ip = {
                    .sin_family = AF_INET,
                    .sin_port = htons(parsed_port),
                };
                if(!inet_pton(AF_INET, ipv4_value, &cb.ip.sin_addr)) {
                    fprintf(stderr, "Cannot parse network state return message - ipv4 address cannot be parsed\n");
                    message->type = PARSE_ERR;
                    return;
                }
                vec_push(&clients_vec, cb);
                client = client.next_sibling();
            }

            message->data.network_r.clients = clients_vec;
            break;
        }
        default: {
            break;
        }
    }
}
