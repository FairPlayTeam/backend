# FairPlay MVP, Backend
The backend rest api for fairplay

➡️ The repository for the **currently running website** ([fairplay.video](https://fairplay.video)) can be found here: [FairPlay-Website\_DEMO](https://github.com/FairPlayTeam/FairPlay-Website_DEMO)
➡️ You can find the frontend code here : [https://github.com/FairPlayTeam/](https://github.com/FairPlayTeam/FairPlay-Website/)

## 🛠 Tech Stack

* **Next.js** for the frontend and website
* **rust** with **axum** for the publicly available rest api

## 🤝 Contribute

Want to participate? Join our development community on Discord:
👉 [discord.gg/fairplayteam](https://discord.gg/fairplayteam)

## Developer TODO List, Prioritized by Importance

### 1. Core Infrastructure & Media Handling

* **Video Storage Management**

  * [ ] Design and implement a scalable storage system for video files.
  * [ ] Support large file uploads (chunked uploads + retry logic).

* **FFmpeg + HLS Encoding Pipeline**

  * [ ] Integrate FFmpeg to transcode uploaded videos into HLS (HTTP Live Streaming).
  * [ ] Generate multiple quality levels (240p, 480p, 720p, 1080p).
  * [ ] Store HLS segments and playlists.

* **MinIO Integration**

  * [ ] Use MinIO (S3-compatible) for storing video files and HLS chunks.
  * [ ] Ensure redundancy and access control (private/public buckets).


### 2. User & Access Management

* **Authentication & Connection System**

  * [ ] Implement secure user login and registration (email/password, OAuth).
  * [ ] Manage sessions or JWT tokens for API access.

* **Private Video Access Control**

  * [ ] Ensure private videos are only accessible by their owner.
  * [ ] Prevent unauthorized access via signed URLs or token-based validation.


### 3. Admin & Moderation Tools

* **Admin & Moderator Dashboard**

  * [ ] Manage videos (view, delete, mark as inappropriate).
  * [ ] Ban users.
  * [ ] View reports and logs.


### 4. Video Platform Features

* **Video Feed (Home / Explore)**

  * [ ] Show a dynamic feed of videos based on user preferences.
  * [ ] Allow sorting & filtering by category, tags, or creator.

* **Offline Videos**

  * [ ] Allow users to download videos for offline access (if allowed by creator).

* **Private Videos**

  * [ ] Enable creators to mark videos as private.
  * [ ] Support custom licenses on videos.

* **YouTube Video Import**

  * [ ] Connect user accounts via OAuth.
  * [ ] Allow importing videos directly from YouTube channels.



### 5. User Interaction & Engagement

* **⭐ Rating System**

  * [ ] Replace like/dislike with a 1–5 star rating system.

* **Subscribe Button + Notifications**

  * [ ] Add subscription options:

    * All notifications
    * Important only
    * Don’t notify me
    * Unsubscribe



### 6. Creator Support & Monetization

* **Support the Creator Button**

  * [ ] Allow users to support creators by:

    * Watching rewarded ads
    * Making a small donation (Stripe, PayPal, etc.)

* **Donate to the Creator Button**

  * [ ] Add a dedicated button for one-time or recurring donations.
