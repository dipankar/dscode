/**
 * NNG Node.js Native Binding
 *
 * Provides bidirectional NNG sockets for extension host:
 * - REP (reply) socket for receiving requests from Tauri
 * - REQ (request) socket for sending requests to Tauri
 * Uses Node-API (N-API) for cross-platform compatibility.
 */

#include <node_api.h>
#include <nng/nng.h>
#include <nng/protocol/reqrep0/rep.h>
#include <nng/protocol/reqrep0/req.h>
#include <string.h>
#include <stdlib.h>
#include <stdbool.h>

// Global socket handles
static nng_socket rep_socket;  // For receiving requests from Tauri
static bool rep_socket_initialized = false;

static nng_socket req_socket;  // For sending requests to Tauri
static bool req_socket_initialized = false;

/**
 * Listen on IPC endpoint
 * JavaScript: listen(url: string) => void
 */
static napi_value Listen(napi_env env, napi_callback_info info) {
    napi_status status;
    size_t argc = 1;
    napi_value args[1];

    status = napi_get_cb_info(env, info, &argc, args, NULL, NULL);
    if (status != napi_ok || argc < 1) {
        napi_throw_error(env, NULL, "Expected 1 argument: url");
        return NULL;
    }

    // Get URL string
    size_t url_len;
    status = napi_get_value_string_utf8(env, args[0], NULL, 0, &url_len);
    if (status != napi_ok) {
        napi_throw_error(env, NULL, "Failed to get URL length");
        return NULL;
    }

    char* url = malloc(url_len + 1);
    status = napi_get_value_string_utf8(env, args[0], url, url_len + 1, &url_len);
    if (status != napi_ok) {
        free(url);
        napi_throw_error(env, NULL, "Failed to get URL string");
        return NULL;
    }

    // Create REP socket
    int rv;
    if (!rep_socket_initialized) {
        rv = nng_rep0_open(&rep_socket);
        if (rv != 0) {
            free(url);
            napi_throw_error(env, NULL, nng_strerror(rv));
            return NULL;
        }
        rep_socket_initialized = true;
    }

    // Listen on the endpoint
    rv = nng_listen(rep_socket, url, NULL, 0);
    free(url);

    if (rv != 0) {
        napi_throw_error(env, NULL, nng_strerror(rv));
        return NULL;
    }

    napi_value result;
    napi_get_undefined(env, &result);
    return result;
}

/**
 * Receive a message (blocking)
 * JavaScript: receive() => string
 */
static napi_value Receive(napi_env env, napi_callback_info info) {
    if (!rep_socket_initialized) {
        napi_throw_error(env, NULL, "REP socket not initialized. Call listen() first.");
        return NULL;
    }

    // Receive message
    char* buf = NULL;
    size_t sz;
    int rv = nng_recv(rep_socket, &buf, &sz, NNG_FLAG_ALLOC);

    if (rv != 0) {
        napi_throw_error(env, NULL, nng_strerror(rv));
        return NULL;
    }

    // Convert to JavaScript string
    napi_value result;
    napi_status status = napi_create_string_utf8(env, buf, sz, &result);
    nng_free(buf, sz);

    if (status != napi_ok) {
        napi_throw_error(env, NULL, "Failed to create string");
        return NULL;
    }

    return result;
}

/**
 * Send a message (reply)
 * JavaScript: send(message: string) => void
 */
static napi_value Send(napi_env env, napi_callback_info info) {
    napi_status status;
    size_t argc = 1;
    napi_value args[1];

    if (!rep_socket_initialized) {
        napi_throw_error(env, NULL, "REP socket not initialized. Call listen() first.");
        return NULL;
    }

    status = napi_get_cb_info(env, info, &argc, args, NULL, NULL);
    if (status != napi_ok || argc < 1) {
        napi_throw_error(env, NULL, "Expected 1 argument: message");
        return NULL;
    }

    // Get message string
    size_t msg_len;
    status = napi_get_value_string_utf8(env, args[0], NULL, 0, &msg_len);
    if (status != napi_ok) {
        napi_throw_error(env, NULL, "Failed to get message length");
        return NULL;
    }

    char* message = malloc(msg_len + 1);
    status = napi_get_value_string_utf8(env, args[0], message, msg_len + 1, &msg_len);
    if (status != napi_ok) {
        free(message);
        napi_throw_error(env, NULL, "Failed to get message string");
        return NULL;
    }

    // Send message
    int rv = nng_send(rep_socket, message, msg_len, 0);
    free(message);

    if (rv != 0) {
        napi_throw_error(env, NULL, nng_strerror(rv));
        return NULL;
    }

    napi_value result;
    napi_get_undefined(env, &result);
    return result;
}

/**
 * Close the REP socket
 * JavaScript: close() => void
 */
static napi_value Close(napi_env env, napi_callback_info info) {
    if (rep_socket_initialized) {
        nng_close(rep_socket);
        rep_socket_initialized = false;
    }

    napi_value result;
    napi_get_undefined(env, &result);
    return result;
}

/**
 * Connect to IPC endpoint (REQ socket)
 * JavaScript: connect(url: string) => void
 */
static napi_value Connect(napi_env env, napi_callback_info info) {
    napi_status status;
    size_t argc = 1;
    napi_value args[1];

    status = napi_get_cb_info(env, info, &argc, args, NULL, NULL);
    if (status != napi_ok || argc < 1) {
        napi_throw_error(env, NULL, "Expected 1 argument: url");
        return NULL;
    }

    // Get URL string
    size_t url_len;
    status = napi_get_value_string_utf8(env, args[0], NULL, 0, &url_len);
    if (status != napi_ok) {
        napi_throw_error(env, NULL, "Failed to get URL length");
        return NULL;
    }

    char* url = malloc(url_len + 1);
    status = napi_get_value_string_utf8(env, args[0], url, url_len + 1, &url_len);
    if (status != napi_ok) {
        free(url);
        napi_throw_error(env, NULL, "Failed to get URL string");
        return NULL;
    }

    // Create REQ socket
    int rv;
    if (!req_socket_initialized) {
        rv = nng_req0_open(&req_socket);
        if (rv != 0) {
            free(url);
            napi_throw_error(env, NULL, nng_strerror(rv));
            return NULL;
        }

        // Set timeout for receives
        nng_duration timeout = 30000; // 30 seconds
        rv = nng_socket_set_ms(req_socket, NNG_OPT_RECVTIMEO, timeout);
        if (rv != 0) {
            nng_close(req_socket);
            free(url);
            napi_throw_error(env, NULL, nng_strerror(rv));
            return NULL;
        }

        req_socket_initialized = true;
    }

    // Connect to the endpoint (dial)
    rv = nng_dial(req_socket, url, NULL, 0);
    free(url);

    if (rv != 0) {
        napi_throw_error(env, NULL, nng_strerror(rv));
        return NULL;
    }

    napi_value result;
    napi_get_undefined(env, &result);
    return result;
}

/**
 * Send a request and wait for response (REQ socket)
 * JavaScript: request(message: string) => string
 */
static napi_value Request(napi_env env, napi_callback_info info) {
    napi_status status;
    size_t argc = 1;
    napi_value args[1];

    if (!req_socket_initialized) {
        napi_throw_error(env, NULL, "REQ socket not initialized. Call connect() first.");
        return NULL;
    }

    status = napi_get_cb_info(env, info, &argc, args, NULL, NULL);
    if (status != napi_ok || argc < 1) {
        napi_throw_error(env, NULL, "Expected 1 argument: message");
        return NULL;
    }

    // Get message string
    size_t msg_len;
    status = napi_get_value_string_utf8(env, args[0], NULL, 0, &msg_len);
    if (status != napi_ok) {
        napi_throw_error(env, NULL, "Failed to get message length");
        return NULL;
    }

    char* message = malloc(msg_len + 1);
    status = napi_get_value_string_utf8(env, args[0], message, msg_len + 1, &msg_len);
    if (status != napi_ok) {
        free(message);
        napi_throw_error(env, NULL, "Failed to get message string");
        return NULL;
    }

    // Send request
    int rv = nng_send(req_socket, message, msg_len, 0);
    free(message);

    if (rv != 0) {
        napi_throw_error(env, NULL, nng_strerror(rv));
        return NULL;
    }

    // Wait for response
    char* response_buf = NULL;
    size_t response_sz;
    rv = nng_recv(req_socket, &response_buf, &response_sz, NNG_FLAG_ALLOC);

    if (rv != 0) {
        napi_throw_error(env, NULL, nng_strerror(rv));
        return NULL;
    }

    // Convert to JavaScript string
    napi_value result;
    status = napi_create_string_utf8(env, response_buf, response_sz, &result);
    nng_free(response_buf, response_sz);

    if (status != napi_ok) {
        napi_throw_error(env, NULL, "Failed to create string");
        return NULL;
    }

    return result;
}

/**
 * Close the REQ socket
 * JavaScript: closeReq() => void
 */
static napi_value CloseReq(napi_env env, napi_callback_info info) {
    if (req_socket_initialized) {
        nng_close(req_socket);
        req_socket_initialized = false;
    }

    napi_value result;
    napi_get_undefined(env, &result);
    return result;
}

/**
 * Module initialization
 */
static napi_value Init(napi_env env, napi_value exports) {
    napi_status status;
    napi_value fn;

    // REP socket functions (for receiving requests from Tauri)
    // Export listen()
    status = napi_create_function(env, NULL, 0, Listen, NULL, &fn);
    if (status != napi_ok) return NULL;
    status = napi_set_named_property(env, exports, "listen", fn);
    if (status != napi_ok) return NULL;

    // Export receive()
    status = napi_create_function(env, NULL, 0, Receive, NULL, &fn);
    if (status != napi_ok) return NULL;
    status = napi_set_named_property(env, exports, "receive", fn);
    if (status != napi_ok) return NULL;

    // Export send()
    status = napi_create_function(env, NULL, 0, Send, NULL, &fn);
    if (status != napi_ok) return NULL;
    status = napi_set_named_property(env, exports, "send", fn);
    if (status != napi_ok) return NULL;

    // Export close()
    status = napi_create_function(env, NULL, 0, Close, NULL, &fn);
    if (status != napi_ok) return NULL;
    status = napi_set_named_property(env, exports, "close", fn);
    if (status != napi_ok) return NULL;

    // REQ socket functions (for sending requests to Tauri)
    // Export connect()
    status = napi_create_function(env, NULL, 0, Connect, NULL, &fn);
    if (status != napi_ok) return NULL;
    status = napi_set_named_property(env, exports, "connect", fn);
    if (status != napi_ok) return NULL;

    // Export request()
    status = napi_create_function(env, NULL, 0, Request, NULL, &fn);
    if (status != napi_ok) return NULL;
    status = napi_set_named_property(env, exports, "request", fn);
    if (status != napi_ok) return NULL;

    // Export closeReq()
    status = napi_create_function(env, NULL, 0, CloseReq, NULL, &fn);
    if (status != napi_ok) return NULL;
    status = napi_set_named_property(env, exports, "closeReq", fn);
    if (status != napi_ok) return NULL;

    return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, Init)
