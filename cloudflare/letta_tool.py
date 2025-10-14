import os

def show_popup(definition: dict, timeout_ms: int = 300000) -> dict:
    """
    Display a native GUI popup window and wait for user response.

    Args:
        definition (dict): Popup definition with 'title' (optional) and 'elements' (required array)
        timeout_ms (int): Timeout in milliseconds (default: 300000 = 5 minutes)

    Returns:
        dict: Popup result with status ('completed', 'cancelled', 'timeout', 'error') and field values

    Environment Variables:
        POPUP_AUTH_TOKEN: Bearer token for authenticating with popup server (required)

    Example:
        result = show_popup({
            "title": "Confirmation",
            "elements": [
                {"type": "text", "content": "Are you sure?"},
                {"type": "checkbox", "label": "Don't ask again"}
            ]
        })
    """
    import requests

    # Load auth token from environment
    auth_token = os.getenv('POPUP_AUTH_TOKEN')
    if not auth_token:
        return {
            "status": "error",
            "message": "POPUP_AUTH_TOKEN environment variable not set"
        }

    host = os.getenv('HOST')

    try:
        response = requests.post(
            f"{host}/popup",
            json={"definition": definition, "timeout_ms": timeout_ms},
            headers={"Authorization": f"Bearer {auth_token}"},
            timeout=(timeout_ms / 1000) + 5,  # Add 5s buffer to request timeout
        )

        result = response.json()
        return result

    except requests.exceptions.Timeout:
        return {"status": "error", "message": "Request timed out"}
    except requests.exceptions.ConnectionError:
        return {"status": "error", "message": f"Cannot connect to popup server at {host}"}
    except Exception as e:
        return {"status": "error", "message": str(e)}
