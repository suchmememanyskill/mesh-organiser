# Mesh Organiser

Competes with your downloads folder for storing models.

![Preview image](./readme/app.png)

## Install

See the [Releases](https://github.com/suchmememanyskill/mesh-organiser/releases) tab for install files.
- For windows users, use the .msi

## Site integrations

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

## Structure breakdown

The app knows 3 layers of organisation:
- Model: A singular 3d model of any kind
- Group: A collection of 3d models with a strong relationship (like multiple parts of a bigger model)
    - Groups their contents are not intended to be edited after creation. Use Labels for this goal.
- Label: A collection of 3d models with a weak relationship (like multiple models/groups of the same type; like 'wall art' or 'puzzle')
    - Labels thier contents can be edited at any time using the label dropdown menu on groups, models or a collection of models.

## Credits

Developed by [Sims](https://github.com/suchmememanyskill)
- With help from [dorkeline](https://github.com/dorkeline) and Ajadaz

Links:
- [Thumbnail Generator](https://github.com/suchmememanyskill/mesh-thumbnail)
- [Report an issue / Request a feature](https://github.com/suchmememanyskill/mesh-organiser/issues)
- [Donate on Ko-Fi](https://ko-fi.com/suchmememanyskill)