# WizardBot

Discord bot in Rust that asks instructions from a [text-generation-webui](https://github.com/oobabooga/text-generation-webui) API.

Set `DISCORD_TOKEN` and `API_URL` in the environment variables or a .env file. 

Example `API_URL` value: `http://localhost:5000/api/v1/chat` (the `text-generation-webui` endpoint)

Run `text-generation-webui` with `--api`, and with`--listen` if you're accessing it from outside localhost.

Discord command to use: `/chat`