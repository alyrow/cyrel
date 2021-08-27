class Template {
    /**
     * @type {(name: string, variables: any, element: HTMLElement, onLoaded: function) => Template}
     * @param name Name of the template file
     * @param variables Variables to inject
     * @param element The element to replace by the template
     * @param onLoaded Function which will be executed when the template is loaded
     */
    constructor(name, variables, element, onLoaded) {
        const xhttp = new XMLHttpRequest();
        xhttp.onreadystatechange = function () {
            if (this.readyState === 4) {
                if (this.status === 200) {
                    let template = this.responseText;
                    let htmlObj = document.createElement("body");
                    Object.keys(variables).forEach(key => {
                        try {
                            template = template.replace(new RegExp("\{\{(" + key + ")\}\}"), variables[key]);
                        } catch (e) {
                            console.error(e);
                        }
                    });

                    //// Can be a dangerous things but since we trust only files in templates folder we can say it's okay
                    htmlObj.innerHTML = template;
                    const elems = htmlObj.getElementsByTagName("*");
                    for (let i = 0; i < elems.length; i++) {
                        const elem = elems[i];
                        const checkFor = elem.getAttribute("for-each");
                        if (checkFor) {
                            let evaluate = "const elementsContainer = document.createElement('div');\n";
                            evaluate += "elementsContainer.innerHtml = ''\n";
                            evaluate += "const " + checkFor + " = variables." + checkFor + ";\n";
                            evaluate += checkFor + ".forEach(";
                            const as = elem.getAttribute("as");
                            if (as) evaluate += as + " => {\n";
                            else evaluate += " () => {\n";
                            const runIf = elem.getAttribute("run-if");
                            if (runIf) evaluate += "if (" + runIf + ") {\n";
                            let _xElement = elem.outerHTML;
                            try {
                                _xElement = _xElement.replace(new RegExp('for-each="\\S*"'), "");
                            } catch (e) {
                                console.error(e);
                            }
                            try {
                                _xElement = _xElement.replace(new RegExp('as="\\S*"'), "");
                            } catch (e) {
                                console.error(e);
                            }
                            try {
                                _xElement = _xElement.replace(new RegExp('run-if="\\S*"'), "");
                            } catch (e) {
                                console.error(e);
                            }

                            evaluate += "let innerHtml = _xElement;\n" +
                                "innerHtml = innerHtml.replaceAll(/{{(\\S*)}}/gm, substring => {return eval(/{{(\\S*)}}/.exec(substring)[1])});\n" +
                                "elementsContainer.innerHtml += innerHtml;\n";

                            if (runIf) evaluate += "}\n";
                            evaluate += "});\n" +
                                "template = template.replace(elem.outerHTML, elementsContainer.innerHtml);";
                            eval(evaluate);
                        }
                    }

                    element.outerHTML = template;

                    onLoaded();
                }
                if (this.status === 404) element.innerHTML = "Template not found.";
            }
        }
        xhttp.open("GET", "templates/" + name + ".html", true);
        xhttp.send();
    }
}
