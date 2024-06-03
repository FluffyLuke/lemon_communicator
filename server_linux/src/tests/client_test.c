#include "../../includes/client.h"
#include <stdio.h>
#include <uv.h>

int main() {
    uv_tcp_t s;
    uv_tcp_init(uv_default_loop(), &s);

    printf("\nCreating clients table\n\n");
    uv_tcp_t s2;
    uv_tcp_init(uv_default_loop(), &s2);

    

    client_table_t t;
    init_client_table(&t, 5);
    printf("Table capaticy: %zu\n", t.len);
    printf("Table used capacity: %zu\n", t.used_len);
    printf("Adding clients to table\n");

    client_t c1, c2, c3;
    init_client(&c1,  &s2);
    init_client(&c2,  &s2);
    init_client(&c3,  &s2);   
    add_client(&t, c1);
    add_client(&t, c2);
    add_client(&t, c3);

    printf("\nGetting ids of first 3 clients...\n\n");
    printf("First client's id: %ld\n", c1.id);
    printf("Second client's id: %ld\n", get_client(&t, 1)->id);
    printf("Third client's id: %ld\n", get_client(&t, 2)->id);

    printf("Table capaticy after adding 3 clients: %zu\n", t.len);
    printf("Table used capacity after adding 3 clients: %zu\n", t.used_len);

    printf("\nAdding new 5 new clients...\n\n");
    client_t c4, c5, c6, c7, c8;
    init_client(&c4,  &s2);
    init_client(&c5,  &s2);
    init_client(&c6,  &s2);
    init_client(&c7,  &s2);
    init_client(&c8,  &s2);
    add_client(&t, c4);
    add_client(&t, c5);
    add_client(&t, c6);
    add_client(&t, c7);
    add_client(&t, c8);

    printf("Table capaticy after adding 5 clients: %zu\n", t.len);
    printf("Table used capacity after adding 5 clients: %zu\n", t.used_len);

    printf("\nGetting id...\n\n");
    printf("First client's id: %ld\n", get_client(&t, 0)->id);
    printf("Second client's id: %ld\n", get_client(&t, 1)->id);
    printf("Third client's id: %ld\n", get_client(&t, 2)->id);
    printf("Forth client's id: %ld\n", get_client(&t,3)->id);
    printf("Fifth client's id: %ld\n", get_client(&t, 4)->id);

    printf("\nRemoving client of index 1 (id = 1) and displaying ids\n\n");
    remove_client_index(&t, 1);
    printf("Table capaticy after client removal: %zu\n", t.len);
    printf("Table used capacity after client removal: %zu\n", t.used_len);

    printf("\nGetting first two clients\n\n");
    printf("First client's id: %ld\n", get_client(&t, 0)->id);
    printf("Second client's id: %ld\n", get_client(&t, 1)->id);
    printf("Third client's id: %ld\n", get_client(&t, 2)->id);
    printf("Forth client's id: %ld\n", get_client(&t,3)->id);
    printf("Fifth client's id: %ld\n", get_client(&t, 4)->id);
    printf("Sixth client's id: %ld\n", get_client(&t, 5)->id);
}