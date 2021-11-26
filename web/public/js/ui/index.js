Api.backend.getMyGroups(myGroups => {
    let group;
    for (let i = 0; i < myGroups.length; i++) {
        if (myGroups[i].referent) {
            group = myGroups[i].id;
            break;
        }
    }

    const isoDate = (iso_str) => {
        if (iso_str.indexOf("Z") === iso_str.length - 1) return iso_str.slice(0, iso_str.length - 1);
        else return iso_str;
    };
    const startToday = new Date();
    const endToday = new Date(startToday);
    endToday.setDate(endToday.getDate() + 1);
    endToday.setHours(0, 0, 0, 0);
    const startTomorrow = new Date(endToday);
    const endTomorrow = new Date(startTomorrow);
    endTomorrow.setDate(endTomorrow.getDate() + 1);

    const validateCourse = (course) => {
        if (course === null) return course;
        if (course.module === null) course.module = course.category;
        if (course.teacher === null) course.teacher = "Inconnu";
        course.start = course.start.getHours() + "h" + (course.start.getMinutes() < 10 ? "0" : "") + course.start.getMinutes();
        course.end = course.end.getHours() + "h" + (course.end.getMinutes() < 10 ? "0" : "") + course.end.getMinutes();
        return course;
    }

    Api.backend.getSchedule(isoDate(startToday.toISOString()), isoDate(endToday.toISOString()), group, today => {
        for (let i = 0; i < today.length; i++) {
            today[i].start = new Date(today[i].start);
            today[i].end = new Date(today[i].end);
        }
        today = today.sort((a, b) => a.end - b.end);
        console.log(today);
        if (today.length > 0) {
            const todayCourses = {
                "now": [
                    validateCourse((startToday >= today[0].start && startToday <= today[0].end) ? today[0] : null)
                ],
                "next": [
                    validateCourse((startToday >= today[0].start && startToday <= today[0].end) ? today[1] : today[0])
                ]
            };
            new Template("today-courses", todayCourses, document.getElementById("today"), () => {
            });
        } else document.getElementById("today").outerHTML = "<h3>Rien aujourd'hui</h3>"
    }, err => {
        $('body')
            .toast({
                class: 'error',
                message: UiCore.translateError(err)
            })
        ;
    });
    Api.backend.getSchedule(isoDate(startTomorrow.toISOString()), isoDate(endTomorrow.toISOString()), group, tomorrow => {
        for (let i = 0; i < tomorrow.length; i++) {
            tomorrow[i].start = new Date(tomorrow[i].start);
            tomorrow[i].end = new Date(tomorrow[i].end);
        }
        tomorrow = tomorrow.sort((a, b) => a.end - b.end);
        console.log(tomorrow);
        if (tomorrow.length > 0) {
            const tomorrowCourses = {
                "start": [
                    validateCourse(tomorrow[0])
                ],
                "end": [
                    validateCourse(tomorrow[tomorrow.length - 1])
                ]
            };
            new Template("tomorrow-courses", tomorrowCourses, document.getElementById("tomorrow"), () => {
            });
        } else document.getElementById("tomorrow").outerHTML = "<h3>Vous n'avez pas cours demain</h3>"
    }, err => {
        $('body')
            .toast({
                class: 'error',
                message: UiCore.translateError(err)
            })
        ;
    });
});
