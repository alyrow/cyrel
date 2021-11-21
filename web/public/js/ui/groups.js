class Groups {
    static join(id) {
        document.getElementById("dimmer").classList.add("active");
        Api.backend.getAllGroups(allGroups => {
            const ids = [];
            const recursive = function (id) {
                allGroups.forEach(group => {
                    if (group.id === id) {
                        ids.push(id);
                        if (group.parent !== null) recursive(group.parent);
                    }
                });
            }
            recursive(id);
            Api.backend.joinGroups(ids, a => document.location.reload(), a => document.location.reload());
        }, err => {
            $('body')
                .toast({
                    class: 'error',
                    message: err.message
                })
            ;
        });
    }
}

Api.backend.getMyGroups(myGroups => {
    if (myGroups.length !== 0) {
        UiCore.registerTag("my-groups", element => {
            new Template("my-groups", {
                "groups": myGroups
            }, element, () => {
                document.getElementById("my").style.display = "block";
                document.getElementById("dimmer").classList.remove("active");
            });
        });
    } else {
        Api.backend.getAllGroups(allGroups => {
            UiCore.registerTag("all-groups", element => {
                new Template("all-groups", {
                    "groups": allGroups
                }, element, () => {
                    document.getElementById("all").style.display = "block";
                    document.getElementById("dimmer").classList.remove("active");
                });
            });
        }, err => {
            $('body')
                .toast({
                    class: 'error',
                    message: err.message
                })
            ;
        });
    }
}, err => {
    $('body')
        .toast({
            class: 'error',
            message: err.message
        })
    ;
});

$('.ui.accordion')
    .accordion()
;
