$('.ui.form')
    .form({
        revalidate: false,
        keyboardShortcuts: false,
        fields: {
            email: {
                identifier: 'email',
                rules: [
                    {
                        type: 'empty',
                        prompt: 'Merci de renseigner votre addresse mail'
                    },
                    {
                        type   : 'email',
                        prompt : 'Merci de renseigner une addresse mail valide'
                    }
                ]
            },
            password: {
                identifier: 'password',
                rules: [
                    {
                        type: 'empty',
                        prompt: 'Merci de renseigner votre mot de passe'
                    },
                    {
                        type: 'minLength[8]',
                        prompt: 'Votre mot de passe doit faire {ruleValue} caractères minimum'
                    }
                ]
            }
        }
    })
;

document.getElementById("login").onclick = () => {
    const jquerySelector = $('.ui.form');
    const elem = document.getElementsByTagName("form")[0];
    jquerySelector.form("validate form");
    if (jquerySelector.form("is valid")) {
        elem.classList.add("loading");
        Api.backend.login(jquerySelector.form("get values").email, jquerySelector.form("get values").password, success => {
            Api.backend.getMyGroups(myGroups => {
                elem.classList.remove("loading");
                elem.classList.add("success");
                if (myGroups.length === 0) document.location.href = "/groups.html";
                else document.location.href = "/index.html";
            }, e => {
                elem.classList.remove("loading");
                elem.classList.add("success");
                document.location.href = "/index.html";
            });
        }, failure => {
            elem.classList.remove("loading");
            elem.classList.add("error");
            jquerySelector.form("add errors", [failure.message]);
            //document.getElementById("login-error").innerText = failure.message;
        });
    }
};
