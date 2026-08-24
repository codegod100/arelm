# example

A second Buck2 cell in this repo that *uses* the `root` (arelm) cell: its
`rust_binary` depends on `root//:arelm-lib`, uses its Relm4 re-export, and
defines its own application ID, UI, and entrypoint.

This is the in-repo shape of a downstream project that maps arelm as a cell
and links the public library. From the repo root:

```sh
buck2 run example//:app
```

Same GTK4 desktop app as `buck2 run //:arelm`; the point is the cell
boundary (`example//:app` → `root//:arelm-lib`) with a consumer-owned UI.
