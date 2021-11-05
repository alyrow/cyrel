class Api {
    static #backend = null;
    #rpc

    /**
     * Create the api to interact with the backend
     * @type {(url: string, fatalError: function) => Api}
     * @param url Url of the backend
     * @param fatalError Function to be called on fatal error
     */
    constructor(url, fatalError) {
        this.#rpc = simple_jsonrpc.connect_xhr(url, {
            onerror: fatalError
        });
    }

    /**
     * Return an instance of the api
     * @type {() => Api}
     */
    static get backend() {
        if (this.#backend === null) this.#backend = new Api("http://127.0.0.1:3030", e => {
            $('body')
                .toast({
                    class: 'error',
                    message: e
                })
            ;
        });
        return this.#backend;
    }

    /**
     * Function which logins a user
     * @type {(email: string, password: string, onSuccess: function, onFailure: function) => void}
     * @param email Username
     * @param password Password
     * @param onSuccess When the server validates the login infos
     * @param onFailure When the server rejects login infos
     */
    login(email, password, onSuccess, onFailure) {
        this.#rpc.call("login", {email: email, password: password})
            .then(res => onSuccess(res))
            .catch(err => onFailure(err));
    }

    /**
     * Function which asks the server if the user is eligible to registration
     * @type {(ldap: number, department: string, email: string, onSuccess: function, onFailure: function) => void}
     * @param ldap Ldap id
     * @param department Department id
     * @param email User email without 'at' and domain part
     * @param onSuccess When the server validates the user
     * @param onFailure When the server rejects the user
     */
    register_1(ldap, department, email, onSuccess, onFailure) {
        this.#rpc.call("register_1", {ldap: ldap, department: department, email: email})
            .then(res => onSuccess(res))
            .catch(err => onFailure(err));
    }

    /**
     * Function which checks if user is human
     * @type {(hash: string, onSuccess: function, onFailure: function) => void}
     * @param hash Verification code
     * @param onSuccess Call a function with user identity
     * @param onFailure Check fail
     */
    register_2(hash, onSuccess, onFailure) {
        this.#rpc.call("register_2", {hash: hash})
            .then(res => onSuccess(res))
            .catch(err => onFailure(err));
    }

    /**
     * Function which registers the user
     * @type {(hash: string, firstname: string, lastname: string, password: string, onSuccess: function, onFailure: function) => void}
     * @param hash Verification code
     * @param firstname Firstname
     * @param lastname Lastname
     * @param password Password
     * @param onSuccess User registered
     * @param onFailure Failed to register user
     */
    register_3(hash, firstname, lastname, password, onSuccess, onFailure) {
        this.#rpc.call("register_3", {hash: hash, firstname: firstname, lastname: lastname, password: password})
            .then(res => onSuccess(res))
            .catch(err => onFailure(err));
    }

    /**
     * Function which retrieve a schedule
     * @type {(start: string, end: string, group: number, onSuccess: function, onFailure: function) => void}
     * @param start Start date of the schedule
     * @param end End date of the schedule
     * @param group Return the schedule associated with the user's group
     * @param onSuccess When the schedule is retrieved
     * @param onFailure When an error occur
     */
    getSchedule(start, end, group, onSuccess, onFailure) {
        this.#rpc.call("schedule_get", {start: start, end: end, group: group})
            .then(res => onSuccess(res))
            .catch(err => onFailure(err));
    }
}
