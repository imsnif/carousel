
![carousel](https://github.com/user-attachments/assets/81084a19-c05f-42c0-b853-53df95c18bf6)

Carousel is a [Zellij](https://github.com/zellij-org/zellij) plugin that lets you mark specific panes so that they appear in a quick-jumplist for easy navigation later.

It helps you keep context in a chaotic working environment with lots of unexpected changes!

## How to run?

At your convenience, choose one of:

1. From inside Zellij: `zellij plugin -f -- https://github.com/imsnif/carousel/releases/download/latest/carousel.wasm`
2. From inside Zellij: open the plugin manager `Ctrl o` + `p`, press `Ctrl a` and paste this URL inside: `https://github.com/imsnif/carousel/releases/download/latest/carousel.wasm`
3. Add this URL to the `load_plugins` section of your configuration so that it loads on startup, eg.
```kdl
load_plugins {
    "https://github.com/imsnif/carousel/releases/download/latest/carousel.wasm"
}
```
