# CYREL

![](./web/public/images/logos/cyrel_long-512.png)

Cyrel is a website combined wih a frontend for the CY Cergy Paris University's students.

It uses the university environment (Celcat) server Api's (not the admin API), to give various information, in particular
the timetable.

It also gives other information thanks to the addition of students. (still in dev)

## Backend

The backend is json-rpc server written in Rust. Its current functionalities are:

- Create users account
- Log a user
- Get if a user is logged
- Assign groups to users
- Get all groups
- Get groups of a user
- Get schedule (display and download)

## Frontend

The frontend is written in html and plain javascript and take advantage of all backend's functionalities.

&nbsp;

🧃
