# Narwhal File Manager
[![Please Don't ship WIP](https://img.shields.io/badge/Please-Don't%20Ship%20WIP-yellow)](https://dont-ship.it/)


## Goals of the project

Narwhal seeks to be a file manager for linux that is fundamentally "the bastard child of ranger and GUI tech". Keyboard driven file management is great, but so is GUI, and by marrying the two we can make a better experience that is simultaneously faster than traditional file management and is more intuitive than keyboard file management, meeting in the middle between the two paradigms.

Performance is king. If it causes stutters on a decade old laptop, I don't want to see it. This is why we cache file mimetypes and use async when fetching icons. They might not be necessary on modern machines, but on machines of this age, it's vital.

## To Do

File Search -- use async

Perhaps a way to edit mimetype associations?

Add more animations, they look pretty

make file picker xdg portal