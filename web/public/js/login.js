$('.ui.form')
    .form({
        fields: {
            username: {
                identifier: 'username',
                rules: [
                    {
                        type: 'empty',
                        prompt: 'Merci de renseigner votre nom d\'utilisateur'
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
        Api.backend.login(jquerySelector.form("get values").username, jquerySelector.form("get values").password, success => {
            elem.classList.remove("loading");
            elem.classList.add("success");
        }, failure => {
            elem.classList.remove("loading");
            elem.classList.add("error");
            jquerySelector.form("add errors", [failure.message]);
            //document.getElementById("login-error").innerText = failure.message;
        });
    }
};
