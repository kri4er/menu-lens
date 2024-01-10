### Menu-lens
 * Telegram bot aimed to help people traveling pick a dish in the restaurant

### Workflow
 * Open the bot in telegram: https://t.me/Menu_LensBot
 * Make a photo of a menu or a text of the dish name
 * -> Bot will respond you with inline keyboard where you can select dish name
 * <- when clicked on the dish name button, you'll be sent to google pictures search with dish name

### TODO
* Add models to support multiple languages:
  * German
  * Spanish
  * French
* Add NLP to clean up the resulting text and get rid of garbage words
* In Chat pictures
* Add dish nutrition and ingredients
* Try to search for pictured with geocode, next to you - increasing a chance getting picture from exact restaurant

### Version history
 * V0.1 - Alpha not stable - just released

### Hosting
 * Currenty hosted in AWS lambda with 10 max concurrent executions(trying to stay in FreeTier)
