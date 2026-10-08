# Farm Doctor: use case diagram (2026-10-08)

## Actors
- **Farmer** (mobile app)
- **Herder** (mobile app)
- **Ministry planner** (web map)
- **Extension / plant-protection officer** (web map; receives escalations)
- **Public** (web map, no login)
- External systems: **Sentinel-2** (ESA), **Open-Meteo** (ECMWF/GraphCast forecasts), **Claude**, **Google Chirp** (Sorani speech), **SAM**, **Prithvi**, **WorldCereal**, **KRSO/WFP data**

## Diagram

```mermaid
flowchart LR
  Farmer([👨‍🌾 Farmer])
  Herder([🐑 Herder])
  Planner([🏛 Ministry planner])
  Officer([🧑‍🔬 Extension officer])
  Public([👥 Public])

  subgraph SYS["FARM DOCTOR — system boundary"]
    direction TB
    subgraph F["Farmer functions"]
      UC1([Select my field<br/>tap once, SAM draws it])
      UC2([See my field from space<br/>Field Eye])
      UC3([Get this week's plan<br/>Weather Planner])
      UC4([Ask the Doctor<br/>voice, text or photos, Sorani])
      UC5([Report a problem on the map<br/>Neighbour Watch])
      UC6([Get damage proof<br/>Damage Proof])
      UC7([Receive alerts<br/>frost, heat, rain, rust, sunn pest])
    end
    subgraph H["Herder functions"]
      UC8([Heat-stress and pasture alerts<br/>Animal Health])
    end
    subgraph M["Ministry functions"]
      UC9([See the region now<br/>season label, field condition, outbreaks])
      UC10([Weekly briefing in Sorani])
      UC11([Review escalated cases])
      UC12([Sown-area and farmland-loss maps])
    end
    subgraph P["Public functions"]
      UC13([Dam Watch and drought status])
      UC14([Market Watch, WFP prices])
    end
  end

  Farmer --> UC1 & UC2 & UC3 & UC4 & UC5 & UC6 & UC7
  Herder --> UC8
  Planner --> UC9 & UC10 & UC12
  Officer --> UC11
  Public --> UC13 & UC14

  UC2 -.->|includes| UC1
  UC4 -.->|includes| UC2 & UC3 & UC5
  UC6 -.->|includes| UC2
  UC9 -.->|includes| UC5
  UC11 -.->|extends: Doctor unsure| UC4

  S2[(Sentinel-2 / MODIS)] --- UC2 & UC6 & UC9 & UC12 & UC13
  OM[(Open-Meteo<br/>AI weather models)] --- UC3 & UC7 & UC8
  CL[(Claude)] --- UC4 & UC10
  CH[(Google Chirp)] --- UC4
  SAM[(SAM)] --- UC1
  PR[(Prithvi)] --- UC6
  WC[(WorldCereal)] --- UC2 & UC12
  KD[(KRSO / WFP)] --- UC14 & UC9
```

## Scope boundary

**Inside the system**
- The 5 core AIs: Field Eye, Weather Planner, Plant Doctor, Season Check, Neighbour Watch.
- The add-ons: Dam Watch, Damage Proof, Farmland Loss, Animal Health, Market Watch, Sown Map.
- The Doctor layer (Claude, Sorani): combines the AIs' numbers into advice, shows which input drove it, refuses doses, escalates when unsure.
- Farmer mobile app, Ministry web map, public web page.
- Case log for measuring accuracy.

**Outside the system (used, not built)**
- Satellites and their data (ESA, NASA), weather models (ECMWF, Google), Claude, Chirp, SAM, Prithvi, WorldCereal, KRSO and WFP statistics.
- Extension officers and vets: the system hands cases to them; it does not replace them.

**Explicitly out of scope**
- Long-range forecasts (season, El Niño, climate).
- Pesticide or fertilizer doses.
- Yield in tonnes per field; soil fertility maps; price forecasts; credit, insurance, fraud detection; food-safety checks from photos.
- Any hardware (sensors, drones).

## Main functions, one line each
| # | Function | Input | Output |
|---|---|---|---|
| UC1 | Select my field | one tap on the map | field boundary (SAM) |
| UC2 | See my field from space | field boundary | satellite picture, greenness vs own normal and vs neighbours, weak patches, since when |
| UC3 | This week's plan | village, crop, growth stage, 10-day forecast | sow / urea / spray / harvest timing, frost and heat warnings |
| UC4 | Ask the Doctor | voice, text, 3–6 photos | likely cause, confidence, what to do now, what it cannot tell, referral |
| UC5 | Report a problem | pin + photo + type | pin on the outbreak map; nearby similar reports |
| UC6 | Damage proof | field + event date | before/after pictures, damaged %, claim letter |
| UC7 | Alerts | push notifications | frost, heat, heavy rain, rust weather, sunn-pest window |
| UC8 | Herder alerts | village | heat-stress index, pasture greenness vs normal |
| UC9 | Region now | — | Kurdistan → area → zone: season label, condition, outbreaks, dams |
| UC10 | Weekly briefing | region data | one-page Sorani/English brief |
| UC11 | Escalations | Doctor "unsure" cases | queue for officers, with all inputs |
| UC12 | Sown area and farmland loss | season / 25 years | maps per district |
| UC13 | Dam Watch | — | Dukan, Darbandikhan area and % of full, drought status |
| UC14 | Market Watch | WFP prices | this month vs last, unusual jumps |
