# Osu!track v2

The WIP successor to [osu!track](https://ameobea.me/osutrack/) re-designed to be feature-packed, fast, robust, and cute.  It's fully open source with a public API giving access to all stored osu!track data.  Built using a Rust backend and utilizing the Rocket webserver, it's highly performant and leagues ahead of the PHP monstrosity I built as my learn-webdev project years ago.

## Installing the Dev Environment
This section will be added once I flesh a bit more of the foundation of the application out.

## Contributing
This is a project for the community, and so the community is welcome to contribute and help make it their own!  I'm going to complete it no matter what levels of contribution the project receives, but any help is very much appreciated.

If you want to contribute, the best place to start is the osu!track v2 development Discord server:

<center><iframe src="https://discordapp.com/widget?id=299018620573450250&theme=light" width="50%" height="475" allowtransparency="true" frameborder="0"></iframe></center>

Ping me (@Ameo) and I'd be more than happy to chat with you about any ideas you have for the site, questions you have about its development, or anything else.  I really want to make this a community-focused project, so I'm happy to implement peoples' ideas for new features or improvements.

I'm happy to work with people of all levels of programming experience.  Osu!track was one of the first projects that I'd ever created, and it wouldn't have been possible without the help of [Redback](https://rdbk.tv/) and [Reese](https://twitter.com/ReeseWasHere) who helped to make the project into what it is today.  Together with projects like [Tillerinobot](https://github.com/Tillerino/Tillerinobot) and the osu! game itself, I want to help make the osu!track community a place for new developers to learn, grow, and 

## Tech Stack
The site is broken up into two parts: the frontend and the backend.  The backend mainly serves as an API platform to process data out of the database and relay it to clients while the frontend serves as the primary interface between users and the site.

### Frontend
The frontend is built using the [dva](https://github.com/dvajs/dva) framework which is built on top of React, Redux, and React-Router.  It's a fully-featured platform for creating responsive React applications, has support for all kinds of cool features, and is named after an Overwatch character.  It goes hand-in-hand with the [roadhog](https://github.com/sorrycc/roadhog) server, which is basically a souped-up version of [create-react-app](https://github.com/facebookincubator/create-react-app).  If you're comfortable working with React and/or Redux, this should seem very familiar.

### Backend
The backend is written in [Rust](https://reddit.com/r/rust) using the [Rocket](https://rocket.rs/) webserver.  I think Rust is an amazing language: fast, safe, clean, and productive to program in.  However, if you've never worked with it before, it has a bad reputation for having a bit of a tough learning curve.

I chose Rust because it's a language I've worked with in the past, it has a really great community, and it's *fast*.  No more waiting 15 seconds for pages to load; I want data to be available almost instantly for all pages and API endpoints.