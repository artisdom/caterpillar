import { JSX } from "@bossley9/sjsx/jsx-runtime";
import * as gfm from "@deno/gfm";

export const dailyThoughtsPage = (dates: string[]) => {
    const entries = [];
    for (const date of dates) {
        entries.push(dailyThoughtItem(date));
    }

    return page(
        "Daily Thoughts",
        <>
            <h2>Daily Thoughts</h2>
            <p>
                Hey, I'm Hanno! These are my daily thoughts on{" "}
                <a href="https://github.com/hannobraun/caterpillar">
                    Caterpillar
                </a>, the programming language I'm creating. If you have any
                questions, comments, or feedback, please{" "}
                <a href="mailto:hello@hannobraun.com">
                    get in touch
                </a>!
            </p>
            <ol class="m-8">{entries}</ol>
        </>,
    );
};

export const singleDailyThoughtPage = (
    date: string,
    md: string,
    dates: string[],
) => {
    const html = gfm.render(md, {
        allowedTags: ["source"],
        allowedAttributes: { "source": ["src"] },
    });

    const index = dates.findIndex((element) => element == date);

    const prev = dates[index + 1];
    const next = dates[index - 1];

    return page(
        `Daily Thought - ${date}`,
        <>
            <h2>Daily Thought - {date}</h2>
            <a href="/daily">{"< "}back to list</a>
            <main class="prose">
                {html}
            </main>
            <div class="grid grid-cols-2">
                {prev && (
                    <span class="col-1 justify-self-start">
                        {dailyThoughtLink(prev, "<< previous thought")}
                    </span>
                )}
                {next && (
                    <span class="col-2 justify-self-end">
                        {dailyThoughtLink(next, "next thought >>")}
                    </span>
                )}
            </div>
        </>,
    );
};

const dailyThoughtItem = (date: string) => {
    const link = dailyThoughtLink(date, date);

    return (
        <li class="my-4 font-bold text-lg">
            {link}
        </li>
    );
};

const dailyThoughtLink = (date: string, label: string) => {
    const link = `/daily/${date}`;

    return (
        <a href={link}>
            {label}
        </a>
    );
};

const page = (title: string, content: JSX.Element) => {
    return (
        <>
            {"<!doctype html>"}
            <html lang="en">
                <head>
                    <title>{title} - Caterpillar</title>

                    <meta charSet="UTF-8" />
                    <meta
                        name="viewport"
                        content="width=device-width, initial-scale=1"
                    />

                    <link href="/style.css" rel="stylesheet" />
                </head>
                <body class="max-w-xl mx-auto p-2">
                    <header>
                        <h1>Caterpillar</h1>
                    </header>
                    <main>
                        {content}
                    </main>
                </body>
            </html>
        </>
    );
};
