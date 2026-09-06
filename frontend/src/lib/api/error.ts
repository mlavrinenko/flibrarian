export class ApiError extends Error {
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }

  /** A 4xx will fail identically until something outside the app changes. */
  get terminal(): boolean {
    return this.status >= 400 && this.status < 500;
  }
}

export async function apiError(
  response: Response,
  fallback: string,
): Promise<ApiError> {
  let message = fallback;
  try {
    const body = (await response.json()) as { error?: string };
    if (body.error) message = body.error;
  } catch {
    // No JSON body on the response; the fallback stands.
  }
  return new ApiError(response.status, message);
}
