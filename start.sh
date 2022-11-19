## Print start msg
echo "  ██████    ▄▄▄█████▓    ▄▄▄          ██▀███     ▄▄▄█████▓    ██▓    ███▄    █      ▄████                         \n▒██    ▒    ▓  ██▒ ▓▒   ▒████▄       ▓██ ▒ ██▒   ▓  ██▒ ▓▒   ▓██▒    ██ ▀█   █     ██▒ ▀█▒                        \n░ ▓██▄      ▒ ▓██░ ▒░   ▒██  ▀█▄     ▓██ ░▄█ ▒   ▒ ▓██░ ▒░   ▒██▒   ▓██  ▀█ ██▒   ▒██░▄▄▄░                        \n  ▒   ██▒   ░ ▓██▓ ░    ░██▄▄▄▄██    ▒██▀▀█▄     ░ ▓██▓ ░    ░██░   ▓██▒  ▐▌██▒   ░▓█  ██▓                        \n▒██████▒▒     ▒██▒ ░     ▓█   ▓██▒   ░██▓ ▒██▒     ▒██▒ ░    ░██░   ▒██░   ▓██░   ░▒▓███▀▒    ██▓     ██▓     ██▓ \n▒ ▒▓▒ ▒ ░     ▒ ░░       ▒▒   ▓▒█░   ░ ▒▓ ░▒▓░     ▒ ░░      ░▓     ░ ▒░   ▒ ▒     ░▒   ▒     ▒▓▒     ▒▓▒     ▒▓▒ \n░ ░▒  ░ ░       ░         ▒   ▒▒ ░     ░▒ ░ ▒░       ░        ▒ ░   ░ ░░   ░ ▒░     ░   ░     ░▒      ░▒      ░▒  \n░  ░  ░       ░           ░   ▒        ░░   ░      ░          ▒ ░      ░   ░ ░    ░ ░   ░     ░       ░       ░   \n      ░                       ░  ░      ░                     ░              ░          ░      ░       ░       ░  \n                                                                                               ░       ░       ░  \n"

## Stop docker and restart the mongodb container
docker-compose down
docker-compose up -d mongodb_room_manager

## Start the websocket main handler
cargo run 