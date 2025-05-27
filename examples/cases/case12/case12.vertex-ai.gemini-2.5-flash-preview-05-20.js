// written by LLM provider: vertex-ai model: gemini-2.5-flash-preview-05-20
async function main(config, params) {
  // Define the HTTP action to check the service URL
  const httpAction = {
    kind: "http",
    method: "GET",
    url: config.check_url,
    timeout: 2, // Set timeout to 2 seconds as specified
  };

  // Execute the HTTP action
  const httpResult = await doAction(httpAction);

  // Check if the HTTP response status is not in the 2xx range
  if (httpResult.status < 200 || httpResult.status >= 300) {
    // If the check fails, define the notification action
    const notifyAction = {
      kind: "notify",
      url: config.lognotify, // Use the FeiShu webhook URL from config
      title: "Service Health Check Alert", // Title for the notification
      message: `Health check for service URL ${config.check_url} failed with status code: ${httpResult.status}.`, // Message for the notification
    };

    // Execute the notification action and return its result
    return await doAction(notifyAction);
  } else {
    // If the check is successful, return the HTTP result
    return httpResult;
  }
}
