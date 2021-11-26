class RegisterSelect {

    constructor(element, conf) {
        new Template("department-select", {
            "departments": conf
        }, element, () => {
            $('#register_1')
                .form({
                    onSuccess: function () {
                        document.getElementById("check").click();
                    },
                    fields: {
                        id: {
                            identifier: 'id',
                            rules: [
                                {
                                    type: 'empty',
                                    prompt: 'Merci de renseigner votre numéro étudiant'
                                },
                                {
                                    type: 'integer',
                                    prompt: 'Merci de renseigner un numéro d\'étudiant valide'
                                },
                                {
                                    type   : 'exactLength[8]',
                                    prompt : 'Un numéro étudiant fait 8 caractères'
                                }
                            ]
                        },
                        department: {
                            identifier: 'department',
                            rules: [
                                {
                                    type: 'empty',
                                    prompt: 'Merci de renseigner votre département'
                                }
                            ]
                        },
                        email: {
                            identifier: 'email',
                            rules: [
                                {
                                    type: 'empty',
                                    prompt: 'Merci de renseigner votre email'
                                },
                                {
                                    type   : 'doesntContain[@]',
                                    prompt : 'Merci de renseigner que la partie avant le "@" de votre email'
                                }
                            ]
                        }
                    }
                })
            ;
        });
    }

    static setEmailDomain(domain) {
        document.getElementById("email-label").innerText = "@" + domain;
    }
}

UiCore.registerTag("register-select", element => {
    Config.loadConfig("departments", conf => {
        new RegisterSelect(element, conf);
    });
});

document.getElementById("check").onclick = () => {
    const jquerySelector = $('#register_1');
    const elem = document.getElementById("register_1");
    jquerySelector.form("validate form");
    if (jquerySelector.form("is valid")) {
        elem.classList.add("loading");
        Api.backend.register_1(parseInt(jquerySelector.form("get values").id), jquerySelector.form("get values").department, jquerySelector.form("get values").email, success => {
            elem.classList.remove("loading");
            $('#form-check')
                .form({
                    onSuccess: function () {
                        document.getElementById("continue").click();
                    },
                    fields: {
                        code: {
                            identifier: 'code',
                            rules: [
                                {
                                    type: 'empty',
                                    prompt: 'Merci de renseigner votre code de vérification'
                                }
                            ]
                        }
                    }
                })
            ;
            document.getElementById("check").style.display = "none";
            document.getElementById("check-code").style.display = "block";
        }, failure => {
            elem.classList.remove("loading");
            jquerySelector.form("add errors", [failure.message]);
        });
    }
};

document.getElementById("continue").onclick = () => {
    const jquerySelector = $('#form-check');
    const elem = document.getElementById("form-check");
    jquerySelector.form("validate form");
    if (jquerySelector.form("is valid")) {
        elem.classList.add("loading");
        Api.backend.register_2(jquerySelector.form("get values").code, success => {
            elem.classList.remove("loading");
            $('#register_2')
                .form({
                    onSuccess: function () {
                        document.getElementById("register").click();
                    },
                    fields: {
                        firstname: {
                            identifier: 'firstname',
                            rules: [
                                {
                                    type: 'empty',
                                    prompt: 'Merci de renseigner votre prénom'
                                }
                            ]
                        },
                        lastname: {
                            identifier: 'lastname',
                            rules: [
                                {
                                    type: 'empty',
                                    prompt: 'Merci de renseigner votre nom'
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
                        },
                        password2: {
                            identifier: 'password2',
                            rules: [
                                {
                                    type   : 'match[password]',
                                    prompt : 'Merci de confirmer votre mot de passe'
                                }
                            ]
                        }
                    }
                })
                .form("set value", "firstname", success.firstname)
                .form("set value", "lastname", success.lastname);

            document.getElementById("continue").style.display = "none";
            document.getElementById("end").style.display = "block";
        }, failure => {
            elem.classList.remove("loading");
            jquerySelector.form("add errors", [failure.message]);
        });
    }
};

document.getElementById("register").onclick = () => {
    const jquerySelector = $('#register_2');
    const elem = document.getElementById("register_2");
    jquerySelector.form("validate form");
    if (jquerySelector.form("is valid")) {
        elem.classList.add("loading");
        Api.backend.register_3($('#form-check').form("get values").code, jquerySelector.form("get values").firstname, jquerySelector.form("get values").lastname, jquerySelector.form("get values").password, success => {
            elem.classList.remove("loading");
            elem.classList.add("success");
            document.location.href = "/login.html";
        }, failure => {
            elem.classList.remove("loading");
            jquerySelector.form("add errors", [failure.message]);
        });
    }
};
