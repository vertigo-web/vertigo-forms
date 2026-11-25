<!-- markdownlint-configure-file { "no-duplicate-heading": { "siblings_only": true } } -->

<!-- markdownlint-disable-next-line first-line-h1 -->
## Unreleased

### Added

* `Modal` - a window over the whole page with a title and content rendered anew each time it opens,
  closed by Escape, its close button or a click beside it; the keyboard stays in it, the page under
  it doesn't scroll, and a screen reader reads it as a dialog named by its title. Closed, it gives
  the focus back to the button that opened it. A `Modal` opened from another closes on its own
  Escape. With `ModalParams` and `RenderModalContent`
* `DragGroupsParams::render_item_aside` - what stands at the end of an item, beside its button,
  f. ex. a button of its own, which the button of an item can't hold; it's told whether the item
  is picked, a mouse or a pen is over it or the keyboard on its button or in it, and till then
  lets the pointer through to the item. With `RenderItemAside` and `DragGroupsParams::item_box_css`
* `TooltipParams::control` - a `Tooltip` on a control, f. ex. a button or what a tap picks up to
  drag: the bubble shows while a mouse or a pen is over it, a press does what the control does and
  hides the bubble till the pointer leaves, and the element is no tab stop of its own, so a tap,
  Enter and Space stay the control's

### Fixed

* `PhotoPopup` left the focus nowhere once closed, so the keyboard started again from the top of
  the page - it gives it back to the thumbnail or button that opened it
* `DragGroups` put the picked item down on Escape pressed anywhere in it, also in a `Modal` opened
  from its header, which stayed open then - it hears Escape on its items and on "here" only
* `DragGroups` left the focus nowhere once the groups changed, f. ex. after Enter on "here" - it
  goes to the item it was on, wherever the item stands then, or to the one which took its place.
  It renders anew only the items of a group which changed, and what stands above and below them
  keeps the focus

### Changed

* `DragGroupsParams` takes the type of the items too, `DragGroupsParams<G, I>` - inferred where
  it's built
* `TooltipParams` has the field `control` - a struct literal without it needs
  `..TooltipParams::default()`
* `PhotoPopup` moves a swiped photo, and `Tooltip` places its bubble, in whole pixels. A float
  written into a style brought the formatting of floats into the app's wasm, about 20 KB of it

## 0.2.4 - 2026-11-16

### Breaking

* `MultiSelect` needs `params` - `params={}` keeps how it worked and looked

### Added

* `Tooltip` - text next to an element, shown on hover, tap or focus, with `TooltipParams`
* `DragGroups` - items moved between groups by dragging, or picked and put with a tap, click or
  keyboard, with `DragGroup`, `DragMove` and `DragGroupsParams`
* `PhotoPopup` - a photo over the whole page, with buttons, arrow keys and swipe going through
  the photos, with `Photo` and `PhotoPopupParams`
* `PhotoGallery` - thumbnails, each a link to its photo, opening it in a `PhotoPopup`
* `MultiSelectParams`: `MultiSelectMode::Exclusive` (a click selects only the clicked option, as in
  `<select multiple>`), a `none_label` button clearing the selection, and CSS of the buttons
* `MultiSelect`: buttons tell whether they are selected in `aria-pressed`
* `StringValue`, `ListValue`, `DictValue`, `MultiValue` and `BoolValue` exported from `form`,
  so a field can be built on a shared `Value`

### Changed

* Updated to vertigo 0.14.4

## 0.2.3 - 2026-10-21

### Changed

* Updated to vertigo 0.14

## 0.2.2 - 2026-10-14

### Added

* `with_stable_loader` and `WithStableLoader` - a loader that renders once, when the resource first becomes ready,
  and keeps the rendered content on later `Loading`/`Error`, so edit forms survive background re-fetches

### Fixed

* Set `type="button"` for buttons inside forms which are not submits
* `ResourceTable`: the error of a failed delete is shown below the row
* `ResourceTable`: a row keeps its state (edit form with what was typed in, error message) when it is rendered anew
  because its item changed or came back, f. ex. after an optimistic update or removal is rolled back
* `ResourceTable`: "Cancel" drops the changes made in the form, the next edit starts from the item

## 0.2.1 - 2026-09-07

### Changed

* Updated to vertigo 0.13

## 0.2.0 - 2026-07-01

### Added

* `ResourceTable`
* `ImageUpload` helper struct to `DropImageFile`
* "Select file" button to `DropImageFile`
* Optional render customization slots to `LoginParams` (`header`, `footer`, `render_username`, `render_password`, `render_submit`)

### Fixed

* Positioning of `SelectSearch`, `Popup` and `MultiDropDown` popups
* `SelectSearch` sorting

### Changed

* Updated to vertigo 0.12

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
