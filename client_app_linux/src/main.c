#include <stdlib.h>
#include <signal.h>
#include <sys/types.h>
#include <unistd.h>
#include <string.h>
#include <math.h>
#include <gtk/gtk.h>

void on_button1_clicked(GtkButton *b) {
    g_print("hello world!\n");
}

static void activate(GtkApplication *app, gpointer user_data) {

    GtkBuilder* builder = gtk_builder_new();
    gtk_builder_add_from_file(builder, "ui/linux_client.ui", NULL);

    GtkWidget* window = GTK_WIDGET(gtk_builder_get_object(builder, "window"));

    gtk_window_set_application(GTK_WINDOW(window), app);
    GObject* button1 = gtk_builder_get_object(builder, "button1");
    g_signal_connect(button1, "clicked", G_CALLBACK(on_button1_clicked), NULL);

    gtk_widget_set_visible(GTK_WIDGET(window), true);

    g_object_unref(builder);
}

int main(int argc, char** argv) {
    GtkApplication *app = gtk_application_new("lemon.communicator", G_APPLICATION_DEFAULT_FLAGS);
    g_signal_connect(app, "activate", G_CALLBACK(activate), NULL);
    int status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);
    return status;
}