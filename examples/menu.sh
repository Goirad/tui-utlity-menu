#!/bin/bash -e

arg=$(target/release/tui-menu "$1")

case "$arg" in
    RUST_TICKET_*)
        xdg-open https://github.com/rust-lang/rust/issues/"${arg#RUST_TICKET_}" \
            && i3-msg workspace 1:
    ;;
    WEBSITE_GOOGLE)
        xdg-open https://google.com \
            && i3-msg workspace 1:
    ;;
    WEBSITE_GITHUB)
        xdg-open https://github.com \
            && i3-msg workspace 1:
    ;;
    WEBSITE_CRATES_IO)
        xdg-open http://crates.io \
            && i3-msg workspace 1:
    ;;
    *)
        echo "Unrecognized option"
        exit 1
    ;;
esac
