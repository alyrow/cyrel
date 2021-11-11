
class Config {
    /**
     * Load a config file
     * @type {(name: string, callback: function) => Config}
     * @param name Name of the config
     * @param callback Function to be called when config is loaded
     */
    constructor(name, callback) {
        const xhttp = new XMLHttpRequest();
        xhttp.onreadystatechange = function () {
            if (this.readyState === 4) {
                if (this.status === 200) {
                    Config.configs[name] = JSON.parse(this.responseText);
                    callback(Config.configs[name]);
                }
                if (this.status === 404) console.error("Config not found.");
            }
        }
        xhttp.open("GET", "config/" + name + ".json", true);
        xhttp.send();
    }

    static configs = {};

    /**
     * Return an loaded config
     * @type {(name: string) => any}
     * @param name Name of the config
     */
    static getConfig(name) {
        return this.configs[name];
    }

    /**
     * Load a config file and call a function with the config as argument
     * @type {(name: string, callback: function) => void}
     * @param name Name of the config to load
     * @param callback Function to be called when config is loaded
     */
    static loadConfig(name, callback) {
        if (!this.configs[name]) new Config(name, callback);
        else callback(this.configs[name]);
    }
}

Config.loadConfig("errors", () => {});
