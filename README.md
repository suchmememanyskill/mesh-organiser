# Mesh Organiser

Competes with your downloads folder for storing 3d models. Available both as a desktop app or as a web server. [A web-based demo is available for testing](https://meshdemo.suchmeme.nl/)

![Preview image](./readme/app.png)

## Install

Download for:
- [Windows](https://github.com/suchmememanyskill/mesh-organiser/releases/download/v2.1.0/Mesh.Organiser_2.1.0_x64_en-US.msi) (.msi)
- Macos
    - [aarch64/Arm based chips (M-line)](https://github.com/suchmememanyskill/mesh-organiser/releases/download/v2.1.0/Mesh.Organiser_2.1.0.dmg)
    - [x64/Intel based chips](https://github.com/suchmememanyskill/mesh-organiser/releases/download/v2.1.0/Mesh.Organiser_2.1.0_x64.dmg)
- Linux
    - [Debian/Ubuntu](https://github.com/suchmememanyskill/mesh-organiser/releases/download/v2.1.0/Mesh.Organiser_2.1.0_amd64.deb) (.deb)
    - [Fedora/RHEL](https://github.com/suchmememanyskill/mesh-organiser/releases/download/v2.1.0/Mesh.Organiser-2.1.0-1.x86_64.rpm) (.rpm)
    - Note: Only slicers installed via flatpak are supported

See the [Releases](https://github.com/suchmememanyskill/mesh-organiser/releases) tab for more downloads and release notes

## Install (Server)

Mesh Organiser is also available to be hosted as a website by making use of a server running Docker. See the [Web folder](./web/README.md) for server installation instructions.

## Support/Donate

If you like this program, consider [boosting the logo's model on makerworld](https://makerworld.com/en/models/1298078-mesh-organiser-logo-cardboard-box#profileId-1329865) for free

If you really like this program, consider donating on [Ko-Fi](https://ko-fi.com/suchmememanyskill)

## Site integrations

Note: To open 'Open in ...' links from these websites, you need to enable them in settings. By default they are disabled to not overwrite any integrations you may currently have.

### [Thingiverse](https://www.thingiverse.com/)
- Redirect 'Open in Cura' to app (see settings)
- Import .zip (models only)
    - Will automatically make group with .zip name

### [Printables](https://www.printables.com)
- Redirect 'Open in PrusaSlicer' (and other slicers) to app (see settings)
    - When using redirect from Printables, the link field is automatically filled
- Import .zip (models only)
    - Will automatically make group with .zip name

### [Makerworld](https://makerworld.com)
- Redirect 'Open in Bambu Studio' to app (see settings)
- Ability to extract both model and thumbnail image (see settings)

## Additional features

- Compresses imported models to save disk space
- Hold Shift/Control to select multiple models or groups at once
- Import .step files (thumbnail generation does not work yet for .step files) (see settings, disabled by default)
- Import .gcode files
- Open slicer after importing from website (see settings, disabled by default)
- Supported slicers: PrusaSlicer, OrcaSlicer, Cura, Bambu Studio
    - Request more via the [Issues tab](https://github.com/suchmememanyskill/mesh-organiser/issues)
- Deduplicates imported models using a hash
    - Importing the same model twice will not duplicate it; it'll be registered as the same model
- Syncing between the desktop and [server/web](./web/README.md) version

## Credits

Developed by [Sims](https://github.com/suchmememanyskill)
- With development help from [dorkeline](https://github.com/dorkeline) and Ajadaz
- With testing help from atomique13, ioan18 and einso

Links:
- [Thumbnail Generator](https://github.com/suchmememanyskill/mesh-thumbnail)
- [Report an issue / Request a feature](https://github.com/suchmememanyskill/mesh-organiser/issues)