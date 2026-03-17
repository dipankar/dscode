{
  "targets": [
    {
      "target_name": "nng_native",
      "sources": [ "src/native/nng_binding.c" ],
      "include_dirs": [
        "<!@(node -p \"require('node-addon-api').include\")"
      ],
      "libraries": [
        "-lnng"
      ],
      "defines": [ "NAPI_DISABLE_CPP_EXCEPTIONS" ]
    }
  ]
}
