# Capping2025
## Team
* Ellie Knapp - Project Manager
* Christian Farrell - Software Engineer
* Nick Longo - Software Engineer
* Peter Arvanitis - Software Engineer
* Ethan Morton - Software Engineer
## Shared Google Drive
[click here](https://drive.google.com/drive/folders/1qaL4QgcQqS9PJ1wRcjRkr2MEaM80OV9i)
## Brightspace
[click here](https://brightspace.marist.edu/d2l/home/57958)
## Project Idea
Travel agent that plans each day during your travel. The website creates an itinerary based on your location. Users may make changes or ask the agent to make changes to the itinerary.
## Build Project (WIP)
In dev we can use react as a backend to make changes to UI/UX, and that should hopefully make development easier.

In prod, we would compile the react files into html/css/js files (or however it works) and just serve them with our backend (axum).


## Database
```docker-compose up -d``` Bring container up

```psql -h localhost -p 5431 -U postgres -d travelagent``` Login to database