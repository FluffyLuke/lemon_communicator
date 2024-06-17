#include "../../libs/mariadb-connector-c/include/mysql.h"
#include <stdbool.h>
#include <stdint.h>
#include <time.h>
#include <uv.h>
#include <uv/unix.h>


int main() {

    MYSQL* conn = mysql_init(0);

    if(mysql_real_connect(
        conn, 
        "localhost", 
        "root", 
        "devel", 
        "devel",
        3306,
        NULL,
        0
    )) {
        printf("Connected to database\n");
    } else {
        fprintf(stderr, "Cannot connect to database!\n");
    }
    mysql_close(conn);
}