<!-- markdownlint-configure-file { "no-duplicate-heading": { "siblings_only": true } } -->

<!-- markdownlint-disable-next-line first-line-h1 -->
## 0.2.0 - Unreleased

### Added

* `ResourceTable`
* `ImageUpload` helper struct to `DropImageFile`
* "Select file" button to `DropImageFile`
* Optional render customization slots to `LoginParams` (`header`, `footer`, `render_username`, `render_password`, `render_submit`)

### Fixed

* Positioning of `SelectSearch`, `Popup` and `MultiDropDown` popups
* `SelectSearch` sorting

## 0.1.3 - 2026-03-05

### Added

* Added dynamic attributes to various components

  (DropImageFile, Popup, PopupOnHover, SearchPanel, TabsHeader, TabsContent, TabsContentMapped)

### Changed

* Updated to vertigo 0.11

## 0.1.2 - 2026-01-05

### Changed

* Updated to vertigo 0.10

## 0.1.1 - 2025-12-04

### Changed

* Updated to vertigo 0.9

### Fixed

* `DictSelect` now generates empty option only if initial value does not match any of provided options

## 0.1.0 - 2025-09-17

### Added

* `Input`, `InputWithButton`,
* `Switch`,
* `Select`, `DictSelect`, `SelectSearch`, `MultiSelect`, `MultiDropDown`
* `SearchPanel`,
* `DropImageFile`,
* `Tabs`,
* `Login`
* `Spinner`
* `WithLoader`
* `Form`
