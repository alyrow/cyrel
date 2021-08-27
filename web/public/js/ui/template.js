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
                    Object.keys(variables).forEach(key => {
                        try {
                            template = template.replace(new RegExp("\{\{(" + key + ")\}\}"), variables[key]);
                        } catch (e) {
                            console.error(e);
                        }
                    });
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
