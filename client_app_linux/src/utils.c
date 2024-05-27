// #include "includes/utils.h"
// #include <stdint.h>
// #include <stdio.h>
// #include <stdlib.h>
// #include <string.h>
// #include <stdbool.h>
// #include <time.h>

// bool enable_debug = false;
// bool enable_warnings = true;
// bool enable_info = true;
// bool enable_errors = true;


// void init_logging() {
//     const char* info = getenv("DISABLE_ERROR");
//     const char* warn = getenv("DISABLE_ERROR");
//     const char* error = getenv("DISABLE_ERROR");
//     const char* debug = getenv("DISABLE_ERROR");
//     if(info != NULL && strcmp(info, "true")) {
//         enable_errors = false;
//     }
//     if(warn != NULL && strcmp(warn, "true")) {
//         enable_warnings = false;
//     }
//     if(error != NULL && strcmp(error, "true")) {
//         enable_info = false;
//     }
//     if(debug != NULL && strcmp(debug, "true")) {
//         enable_debug = true;
//     }
// }

// void log_message(const char* message, enum LoggingLevel level) {
//     switch (level) {
//         case INFO:
//             if(enable_info) {
//                 printf("INFO: %s", message);
//             }
//             break;
//         case WARNING:
//             if(enable_warnings) {
//                 printf("WARNING: %s", message);
//             }
//             break;
//         case ERROR:
//             if(enable_errors) {
//                 printf("ERROR: %s", message);
//             }
//             break;
//         case DEBUG:
//             if(enable_debug) {
//                 printf("DEBUG %s", message);
//             }
//             break;
//     } 
// }


// // typedef struct lemon_array {
// //     void* array_p;
// //     size_t capacity;
// //     size_t used;
// //     size_t size_of_element;
// // } lemon_array;

// // lemon_array new_lemon_array(size_t size_of_element, size_t init_capacity) {
// //     void* p = malloc(size_of_element * init_capacity);
// //     lemon_array a = {
// //         .array_p = p,
// //         .capacity = init_capacity,
// //         .size_of_element = size_of_element
// //     };
// //     return a;
// // }

// // bool add_to_array(lemon_array array, void * element) {
// //     if(array.capacity <= array.used) {
// //         realloc(array.array_p, (array.size_of_element*10)+array.capacity);
// //     }
// //     array.array_p[array.used] = 
// // }

// // void free_lemon_array(lemon_array array) {
// //     free(array.array_p);
// // }