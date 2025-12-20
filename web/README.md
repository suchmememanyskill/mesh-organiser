# Mesh Organiser Web

Mesh Organiser Web is a web (runs in browser) version of the desktop application, implementing extra features like multi-user support, shares and sync to desktop.

## Setup

Mesh organiser web makes use of the Docker Engine. See [the installation page on the Docker docs](https://docs.docker.com/engine/install/) for how to install Docker on your server if you have not done so already. Mesh Organiser web supports both x86_64 and aarch64 hosts.

Mesh organiser web expects you to route it through a reverse proxy to serve to the public. Please refer to for example [Traefik](https://doc.traefik.io/traefik/) or [Ngnix](https://nginx.org/en/).

**compose.yml**
```yml
services:
  mesh-web:
    image: ghcr.io/suchmememanyskill/mesh-organiser:latest
    container_name: mesh-organiser
    environment:
      - APP_CONFIG_PATH=/cfg/config.json
      #- LOCAL_ACCOUNT_PASSWORD=${LOCAL_ACCOUNT_PASSWORD}
      #- SERVER_PORT=3000
    volumes:
      - ${DATA}:/cfg
    ports:
      - 7834:3000
    restart: unless-stopped
networks: {}
```

**.env**
```ini
LOCAL_ACCOUNT_PASSWORD=changeme
DATA=/path/to/storage/folder
```

Create a folder on your host with a `compose.yml` file and `.env` file, containing the contents above. Inside your .env file, change the `LOCAL_ACCOUNT_PASSWORD` variable to anything else (if you need ideas, run `openssl rand -hex 64`). Change the `DATA` variable a path on your host where the data of mesh organiser will live.

Mesh organiser web makes use of a local account as entrypoint for server admins. It is important you do not use the default local account for model organisation. Use the local account only to create and/or manage other accounts in the settings. If you want to give access to user management to other accounts, under the user edit menu you will find an `Admin user` flag.

If you leave LOCAL_ACCOUNT_PASSWORD commented out, the password will randomise each session of the local user. Check your docker compose logs for the password this session. If you prefer the password to stay permanent, uncomment LOCAL_ACCOUNT_PASSWORD (this will start using the value defined in your .env file).

After you started the mesh organiser web server (with for example `docker compose up -d`), navigate to where the server is hosted in a web browser. You will be greeted with a login screen. Log in with the following credentials:
- Username: `local@noemail.com`
- Password: (password you either got from the docker compose logs or defined in your .env file)

Create a new user in the settings using the local account. Optionally edit this new user and give it the `Admin user` flag. Then log out of the local account, and into your newly created account. You are now ready to use Mesh Organiser Web!

### Environment variables

Key|Value|Default|Required
---|---|---|---
APP_CONFIG_PATH|Path to the server configuration|-|Yes
LOCAL_ACCOUNT_PASSWORD|Password for the `local@noemail.com` account|Random key|No
SERVER_PORT|Port to host Mesh Organiser Web on|3000|No

### Configuration

After booting the server, in your data folder will be a config.json. This file allows configuration of the behaviour of the server. Not all parameters are relevant, as the same configuration file is shared with the desktop version. Below will only contain relevant parameters:

```json
{
    // Path to data folder. Default is the same folder as your config.json file.
    "data_path": "",
    // Color of generated thumbnails on your server. These thumbnails will be shared across all users, regardless of their own theme or preview settings.
    "thumbnail_color": "#EEEEEE",
    // Allow/Disallow importing .step files. Note: File sync will break when this feature is disabled.
    "allow_importing_step": true,
    // If a .3mf thumbnail failed to generate for whatever reason, fall back on the included thumbnail.
    "fallback_3mf_thumbnail": true,
    // If a .3mf contains a pre-generated thumbnail, use this image instead of generating an image of the model.
    "prefer_3mf_thumbnail": true,
    // Max amount of parralel processes this server spawns synchronously. 
    "core_parallelism": 4,
    // Allow/Disallow importing .gcode files. Note: File sync will break when this feature is disabled.
    "allow_importing_gcode": true,
    // If a .gcode file contains a pre-generated thumbnail, use it.
    "prefer_gcode_thumbnail": true,
    // Specify rotation of the generated thumbnails [X, Y, Z]. X rotation moves the camera side to side (with +X moving right), Y rotation moves the camera up and down (with +Y moving up), Z rotation spins the camera (with +Z spinning clockwise).
    "thumbnail_rotation": [
        35,
        30,
        0
    ]
}
```