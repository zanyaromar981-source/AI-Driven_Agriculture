# AI-in-Agriculture Research Library (built 2026-10-08)

**What's inside**
- **96,941 research papers**: every paper on Semantic Scholar from 2000 to 2026 whose title or abstract has an AI term ("artificial intelligence", "machine learning", "deep learning", "neural network", "computer vision", "large language model", "random forest") **and** a farming term (agriculture, crop, farm, farmer, wheat, irrigation, agronomy, livestock …).
- 76,622 papers have an abstract, and 48,855 have a free full-text PDF.
- `metadata/papers.jsonl`: the raw records (title, abstract, year, citations, venue, authors, DOI, links).
- `library.sqlite`: a searchable copy with full-text search and topic tags (wheat, drought, satellite, yield_forecast, disease_pest, irrigation, weather_climate, advisory, llm_chatbot, smallholder, middle_east, impact_evidence, review, livestock, robotics_hardware).
- `pdfs/`: 13,793 free full texts (25 GB), downloaded most-relevant first on 2026-10-08. 35,062 links failed (publishers block scripted downloads); `pdfs/_failed.tsv` lists them with the reason. These are downloaded files: open them in a PDF reader only.

**Search examples** (Terminal):
```
sqlite3 library.sqlite "select year, citations, title from papers where rowid in (select rowid from fts where fts match 'wheat AND drought AND sentinel') order by citations desc limit 20"
sqlite3 library.sqlite "select year, citations, title, oa_pdf from papers where tags like '%middle_east%' and tags like '%wheat%' order by citations desc limit 20"
```

**Scripts**
- `harvest_papers.py`: collects the records.
- `build_library.py`: builds the searchable library.
- `download_pdfs.py [workers] [max]`: fetches the free PDFs; you can stop and resume it.

**What it does NOT contain**: paywalled full texts (only their records and links), and news or blog articles.
