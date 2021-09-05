class Edt {
    svg;
    id;

    /**
     * Create a schedule
     * @type {(element: HTMLElement, lines: number, spacing: number, dayHeight: number, dayLength: number, margin: number, theme: any) => Edt}
     * @param element Edt dom element
     * @param lines Number of lines for time separation (1 line = 30 minutes)
     * @param spacing Space between time
     * @param dayHeight Start position of days
     * @param dayLength Height of a day
     * @param margin Margin of days
     * @param theme Theme to be applied
     */
    constructor(element, lines, spacing, dayHeight, dayLength, margin, theme) {
        if (!element.id)
            element.id = (Math.random() * 9992354).toString(36);
        this.id = element.id;
        this.svg = this.#drawTable(element.id, lines, spacing, dayHeight, dayLength, margin, theme);
    }

    /**
     * Set the schedule
     * @type {(courses: any[]) => void}
     * @param courses Array of course
     */
    setEdt(courses) {
        for (let i = 0; i < document.getElementById(this.id).getElementsByTagName("svg").length; i++) {
            document.getElementById(this.id).removeChild(document.getElementById(this.id).getElementsByTagName("svg")[i]);
        }
        this.svg = this.#drawTable(this.id, this.svg._lines, this.svg._spacing, this.svg._dayHeight, this.svg._dayLength, this.svg._margin, this.svg._theme);
        const svg = document.getElementsByTagName("svg")[0];
        svg.removeAttribute('height');
        svg.setAttribute("width", "100%");
        svg.setAttribute("viewBox", "0 0 "+this.svg._width+" "+this.svg._height+"");
        courses.forEach(course => {
            const start = new Date(course.start);
            const end = new Date(course.end);
            const name = course.module? course.module: (course.description? course.description: (course.category? course.category: ""));
            const teacher = course.teacher? course.teacher: "";
            this.#drawCourse(this.svg, name, this.svg._days[start.getDay() - 1], start, end, teacher, this.#colorEvent(course.category, this.svg._theme));
        });
    }

    /**
     * Return a color associated with an event type
     * @type {(event: string, theme: any) => string}
     * @param event Event name
     * @param theme Theme
     */
    #colorEvent(event, theme) {
        switch (event) {
            case "TD": return theme.td
            case "TP": return theme.tp
            case "CM": return theme.cm
            case "Examens": return theme.exam
            case "Tiers temps": return theme.tiers
            default: return "#1e7b91"
        }
    }

    /**
     * Draw a blank schedule
     * @type {(id: string, lines: number, spacing: number, dayHeight: number, dayLength: number, margin: number, theme: any) => any}
     * @param id Id of the dom element to add the svg
     * @param lines Number of lines for time separation (1 line = 30 minutes)
     * @param spacing Space between time
     * @param dayHeight Start position of days
     * @param dayLength Height of a day
     * @param margin Margin of days
     * @param theme Theme to be applied
     */
    #drawTable(id, lines, spacing, dayHeight, dayLength, margin, theme) {
        //23
        let days = ["Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi"];
        let width = margin + 3.5 * 16 + dayLength * days.length + margin,
            height = (lines - 1) * spacing + 2 * margin + dayHeight;
        let draw = SVG(id).size(width, height).fill(theme.text);
        draw._width = width;
        draw._height = height;
        draw._lines = lines;
        draw._spacing = spacing;
        draw._dayHeight = dayHeight;
        draw._dayLength = dayLength;
        draw._margin = margin;
        draw._theme = theme;
        draw._days = days;
        draw.rect(width, height).fill(theme.background);
        draw.line(margin, margin, margin + 3.5 * 16 + dayLength * days.length, margin).stroke({
            width: 1,
            color: theme.primary
        });
        for (let i = 0; i < days.length; i++) {
            draw.text(days[i]).move(margin + 3.5 * 16 + dayLength * i + dayLength / 2, margin + 16 - 1).font("anchor", "middle");
            draw.line(margin + 3.5 * 16 + dayLength * (i + 1), margin, margin + 3.5 * 16 + dayLength * (i + 1), (lines - 1) * spacing + margin + dayHeight).stroke({
                width: 1,
                color: theme.primary
            });
        }

        let heure = new Date(2018, 8, 22, 8, 0, 0);
        for (let i = 0; i < lines - 1; i++) {
            draw.line(margin, i * spacing + margin + dayHeight, margin + 3.5 * 16 + dayLength * days.length, i * spacing + margin + dayHeight).stroke({
                width: 1,
                color: !(i % 3) ? theme.primary : theme.secondary
            });
            draw.text(heure.getHours() + "h" + heure.getMinutes() + (heure.getMinutes() === 0 ? "0" : "")).move(margin + 3, i * (spacing) + margin + 16 / 2 - 1 + dayHeight)
            heure.setMinutes(heure.getMinutes() + 30);
        }
        draw.line(margin, (lines - 1) * spacing + margin + dayHeight, margin + 3.5 * 16 + dayLength * days.length, (lines - 1) * spacing + margin + dayHeight).stroke({
            width: 1,
            color: theme.primary
        })

        draw.line(margin, margin, margin, (lines - 1) * spacing + margin + dayHeight).stroke({
            width: 1,
            color: theme.primary
        });
        draw.line(3.5 * 16, margin, 3.5 * 16, (lines - 1) * spacing + margin + dayHeight).stroke({
            width: 1,
            color: theme.primary
        });
        return draw;
    }

    /**
     * Add event to a schedule
     * @type {(draw: any, name: string, day: string, start: Date, end: Date, teacher: string, color: string) => void}
     * @param draw The schedule {@link drawTable}
     * @param name Name of the event
     * @param day Day of the event
     * @param start Start date of the event
     * @param end End date of the event
     * @param teacher Teacher
     * @param color Color of the event
     */
    #drawCourse(draw, name, day, start, end, teacher, color) {
        let x1, x2;
        let i = draw._days.findIndex((element) => element === day);
        x1 = draw._margin + 3.5 * 16 + draw._dayLength * (i);
        x2 = draw._margin + 3.5 * 16 + draw._dayLength * (i + 1) - 1;
        let timeStart = (start.getHours() - 8) * 2 + start.getMinutes() / 30;
        let timeEnd = (end.getHours() - 8) * 2 + end.getMinutes() / 30;
        let y1 = timeStart * draw._spacing + draw._margin + draw._dayHeight;
        let y2 = timeEnd * draw._spacing + draw._margin + draw._dayHeight;
        //i * spacing + margin + dayHeight
        draw.rect(x2 - x1, y2 - y1).move(x1 + 0.1, y1).fill(this.#colorMixer(this.#hexToRgb(color), this.#hexToRgb(draw._theme.background), 0.2));
        draw.line(x1, y1, x2, y1).stroke({
            width: 2,
            color: color
        });
        draw.line(x1, y1, x1, y2).stroke({
            width: 2,
            color: color
        });
        draw.line(x2, y1, x2, y2).stroke({
            width: 2,
            color: color
        });
        draw.line(x1, y2, x2, y2).stroke({
            width: 2,
            color: color
        });

        draw.text(start.getHours() + "h" + start.getMinutes() + (start.getMinutes() === 0 ? "0" : "")).move(x1 + 3, y1 + 3).font("size", 13);
        let mat = draw.text(name).move(x1 + 6 * 7, y1 + 3).font("size", 14).font("weight", 1);

        do {
            mat.text(name);
            if (mat.length() >= (x2 - (x1 + 6 * 7))) {
                let array = name.split("");
                array[array.length - 1] = null;
                array[array.length - 2] = '…';
                name = array.join("");
            }
        } while (mat.length() >= (x2 - (x1 + 6 * 7)));
        if (timeEnd - timeStart > 1) {
            draw.text(end.getHours() + "h" + end.getMinutes() + (end.getMinutes() === 0 ? "0" : "")).move(x1 + 3, y2 + 3 - 16).font("size", 13);
            draw.text((teacher != null) ? teacher.split(", ").join("\n") : "").move(x1 + 3, y1 + 4 + 16).font("size", 14).fill("#00db6b");
        }
    }

    /**
     * Mix two colors with a given factor
     * @type {(colorChannelA: number, colorChannelB: number, amountToMix: number) => number}
     * @param colorChannelA Color one
     * @param colorChannelB Color two
     * @param amountToMix Amount of the mixing
     */
    #colorChannelMixer(colorChannelA, colorChannelB, amountToMix) {
        const channelA = colorChannelA * amountToMix;
        const channelB = colorChannelB * (1 - amountToMix);
        return parseInt(channelA + channelB);
    }

    /**
     * Mix two colors in form of an {@link Number} {@link Array} with a given factor
     * @type {(rgbA: number[], rgbB: number[], amountToMix: number) => string}
     * @param rgbA Array of color one
     * @param rgbB Array of color two
     * @param amountToMix Amount of the mixing
     */
    #colorMixer(rgbA, rgbB, amountToMix) {
        const r = this.#colorChannelMixer(rgbA[0], rgbB[0], amountToMix);
        const g = this.#colorChannelMixer(rgbA[1], rgbB[1], amountToMix);
        const b = this.#colorChannelMixer(rgbA[2], rgbB[2], amountToMix);
        return "rgb(" + r + "," + g + "," + b + ")";
    }

    /**
     * Convert hex color to a rgb color array
     * @type {(hex: string) => number[] | null}
     * @param hex Hex color
     */
    #hexToRgb(hex) {
        const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
        return result ? [
            parseInt(result[1], 16),
            parseInt(result[2], 16),
            parseInt(result[3], 16)
        ] : null;
    }

    static material = {
        "primary": "#000000",
        "secondary": "rgba(0,0,0,0.5)",
        "text": "rgba(0,0,0,0.87)",
        "background": "#fafafa",
        "td": "#4A4AFF",
        "cm": "#FF0000",
        "tp": "#FE8BAD",
        "exam": "#00FFFF",
        "tiers": "#6FFFFF"
    };

    /**
     * Enum of edt container states
     * @type {{READY: number, LOADING: number, ERROR: number}}
     */
    static edtContainerState = {
        READY: 0,
        LOADING: 1,
        ERROR: 2
    }

    /**
     * Set the edt container state
     * @type {(state: Edt.edtContainerState) => void}
     * @param state State to set
     */
    static setEdtContainerState(state) {
        switch (state) {
            case this.edtContainerState.READY:
                document.getElementById("edt-dimmer").classList.remove("active");
                break;
            case this.edtContainerState.LOADING:
                document.getElementById("edt-dimmer").classList.add("active");
                document.getElementById("edt-loader").classList.remove("disabled");
                break;
            case this.edtContainerState.ERROR:
                document.getElementById("edt-dimmer").classList.add("active");
                document.getElementById("edt-loader").classList.add("disabled");
                break;
        }
    }
}

UiCore.registerTag("edt", element => {
    const edt = new Edt(element, 23, 30, 45, 230, 1, Edt.material);
    const svg = document.getElementsByTagName("svg")[0];
    svg.removeAttribute('height');
    svg.setAttribute("width", "100%");
    svg.setAttribute("viewBox", "0 0 "+edt.svg._width+" "+edt.svg._height+"");

    const onSelect = function(date) {
        Edt.setEdtContainerState(Edt.edtContainerState.LOADING);
        let startDate = new Date(date);
        startDate.setHours(0, 0, 0, 0);
        let endDate = new Date(startDate);
        const diffMonday = startDate.getDay() - 1;
        const diffSaturday = 6 - endDate.getDay();
        startDate.setHours(-24 * diffMonday);
        endDate.setHours(24 * diffSaturday);
        const isoDate = (iso_str) => {
            if (iso_str.indexOf("Z") === iso_str.length - 1) return iso_str.slice(0, iso_str.length - 1);
            else return iso_str;
        };
        Api.backend.getSchedule(isoDate(startDate.toISOString()), isoDate(endDate.toISOString()), "1B01A1PRSA", reponse => {
            Edt.setEdtContainerState(Edt.edtContainerState.READY);
            console.log(reponse)
            edt.setEdt(reponse);
        }, err => {
            Edt.setEdtContainerState(Edt.edtContainerState.ERROR);
            $('body')
                .toast({
                    class: 'error',
                    message: err.message
                })
            ;
        });
    };

    onSelect(new Date());

    $('#calendar')
        .calendar({
            type: 'date',
            firstDayOfWeek: 1,
            text: {
                days: ['D', 'L', 'M', 'M', 'J', 'V', 'S'],
                months: ['Janvier', 'Février', 'Mars', 'Avril', 'Mai', 'Juin', `Juillet`, 'Août', 'Septembre', 'Octobre', 'Novembre', 'Decembre'],
                monthsShort: ['Jan', 'Fev', 'Mar', 'Avr', 'Mai', 'Juin', 'Juil', 'Aou', 'Sep', 'Oct', 'Nov', 'Dec'],
                today: 'Aujourd\'hui',
                now: 'Maintenant',
                am: 'AM',
                pm: 'PM'
            },
            disabledDaysOfWeek: [6, 0],
            onSelect: onSelect
        })
    ;
});

