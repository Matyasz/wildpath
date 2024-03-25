# wildpath
A rust library and CLI to resolve wildcards in filepaths on UNIX systems.

The library exposes one method, `resolve`, which will take a filepath with wildcards and return everything on the filesystem that fits the given description.

For example, consider a file system that looks like the following:
```
root
├── blogs
│   ├── blog_1
│   │   ├── assets
│   │   │   └── logo.jpeg
│   │   └── post.txt
│   └── blog_2
│       ├── assets
│       │   └── research_notes.txt
│       └── post.txt
└── videos
    ├── video_1
    │   ├── assets
    │   │   └── logo.jpeg
    │   └── script.txt
    ├── video_2
    │   ├── assets
    │   │   └── sound_effect.wav
    │   └── script.txt
    └── video_3
        ├── assets
        │   └── new_logo.png
        └── script.txt
```
If you want to find all of the text for the content, you can call

```rust
resolve(Path::from("/root/*/*/*.txt"))
```
which will return all of the `txt` files that are 3 layers deep:

```rust
[
    "/root/blogs/blog_1/post.txt",
    "/root/blogs/blog_2/post.txt",
    "/root/videos/video_1/script.txt",
    "/root/videos/video_2/script.txt",
    "/root/videos/video_3/script.txt"
]
```

If you want to see all of the versions of logos that have been used, you can call 

```rust
resolve(Path::from("/root/*/*/assets/*logo*"))

```
which will return 
```rust
[
    "/root/blogs/blog_1/assets/logo.jpeg",
    "/root/videos/video_1/assets/logo.jpeg",
    "/root/videos/video_3/assets/new_logo.png"
]
```

Or if you want to see the logos used for the original blog and video, you can call 
```rust
resolve(Path::from("/root/*/*_1/assets/*logo*"))
```

which will return

```rust
[
    "/root/blogs/blog_1/assets/logo.jpeg",
    "/root/videos/video_1/assets/logo.jpeg"
]
```

and if you add a new directory called `presentations`, as long as it follows the same structure as the others, this same line of code will also pick up the logo used in the first presentation.

### Symlinks
`wildpath` will also follow symlinks and resolve. For example, if you have the following filesystem:

```
root
├── A
│   └── X (<- symlink to B)
└── B
    └── c.png
```
and pass a path of `/root/A/*/*`, you will receive `[/root/B/c.png]` as the output.

### Limitations
- `wildpath` has only been tested on a UNIX system. For now, it may not work on Windows.
- It will also not respect `.` or `..` in a path. For now, these will need to be resolved prior to using the library.