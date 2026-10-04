import type { SearchCriteria } from "../types";

/**
 * Convert a structured SearchCriteria object into a search query string.
 */
export function criteriaToQuery(c: SearchCriteria): string {
  const parts: string[] = [];

  if (c.text?.trim()) {
    parts.push(c.text.trim());
  }
  if (c.prompt?.trim()) {
    const val = c.prompt.trim();
    parts.push(val.includes(" ") ? `prompt:"${val}"` : `prompt:${val}`);
  }
  if (c.negative_prompt?.trim()) {
    const val = c.negative_prompt.trim();
    parts.push(val.includes(" ") ? `neg:"${val}"` : `neg:${val}`);
  }
  if (c.model_name?.trim()) {
    const val = c.model_name.trim();
    parts.push(val.includes(" ") ? `model:"${val}"` : `model:${val}`);
  }
  if (c.model_hash?.trim()) {
    const val = c.model_hash.trim();
    parts.push(val.includes(" ") ? `hash:"${val}"` : `hash:${val}`);
  }
  if (c.sampler?.trim()) {
    const val = c.sampler.trim();
    parts.push(val.includes(" ") ? `sampler:"${val}"` : `sampler:${val}`);
  }

  // Steps
  if (c.min_steps != null && c.max_steps != null) {
    if (c.min_steps === c.max_steps) {
      parts.push(`steps:${c.min_steps}`);
    } else {
      parts.push(`steps:${c.min_steps}..${c.max_steps}`);
    }
  } else if (c.min_steps != null) {
    parts.push(`steps:>=${c.min_steps}`);
  } else if (c.max_steps != null) {
    parts.push(`steps:<=${c.max_steps}`);
  }

  // CFG
  if (c.min_cfg != null && c.max_cfg != null) {
    if (c.min_cfg === c.max_cfg) {
      parts.push(`cfg:${c.min_cfg}`);
    } else {
      parts.push(`cfg:${c.min_cfg}..${c.max_cfg}`);
    }
  } else if (c.min_cfg != null) {
    parts.push(`cfg:>=${c.min_cfg}`);
  } else if (c.max_cfg != null) {
    parts.push(`cfg:<=${c.max_cfg}`);
  }

  // Rating
  if (c.min_rating != null && c.max_rating != null) {
    if (c.min_rating === c.max_rating) {
      parts.push(`rating:${c.min_rating}`);
    } else {
      parts.push(`rating:${c.min_rating}..${c.max_rating}`);
    }
  } else if (c.min_rating != null) {
    parts.push(`rating:>=${c.min_rating}`);
  } else if (c.max_rating != null) {
    parts.push(`rating:<=${c.max_rating}`);
  }

  // Aesthetic
  if (c.min_aesthetic != null && c.max_aesthetic != null) {
    if (c.min_aesthetic === c.max_aesthetic) {
      parts.push(`aesthetic:${c.min_aesthetic}`);
    } else {
      parts.push(`aesthetic:${c.min_aesthetic}..${c.max_aesthetic}`);
    }
  } else if (c.min_aesthetic != null) {
    parts.push(`aesthetic:>=${c.min_aesthetic}`);
  } else if (c.max_aesthetic != null) {
    parts.push(`aesthetic:<=${c.max_aesthetic}`);
  }

  // Favorite
  if (c.is_favorite === true) {
    parts.push("fav:true");
  } else if (c.is_favorite === false) {
    parts.push("fav:false");
  }

  // NSFW
  if (c.is_nsfw === true) {
    parts.push("is:nsfw");
  } else if (c.is_nsfw === false) {
    parts.push("is:sfw");
  }

  // Media Type
  if (c.media_type) {
    parts.push(`type:${c.media_type}`);
  }

  // Duration
  if (c.min_duration != null && c.max_duration != null) {
    if (c.min_duration === c.max_duration) {
      parts.push(`duration:${c.min_duration}`);
    } else {
      parts.push(`duration:${c.min_duration}..${c.max_duration}`);
    }
  } else if (c.min_duration != null) {
    parts.push(`duration:>=${c.min_duration}`);
  } else if (c.max_duration != null) {
    parts.push(`duration:<=${c.max_duration}`);
  }

  // FPS
  if (c.min_fps != null && c.max_fps != null) {
    if (c.min_fps === c.max_fps) {
      parts.push(`fps:${c.min_fps}`);
    } else {
      parts.push(`fps:${c.min_fps}..${c.max_fps}`);
    }
  } else if (c.min_fps != null) {
    parts.push(`fps:>=${c.min_fps}`);
  } else if (c.max_fps != null) {
    parts.push(`fps:<=${c.max_fps}`);
  }

  return parts.join(" ");
}

/**
 * Count non-empty filter criteria (excluding general text, sorting, and pagination).
 */
export function countActiveFilters(c: SearchCriteria): number {
  let count = 0;
  if (c.prompt?.trim()) count++;
  if (c.negative_prompt?.trim()) count++;
  if (c.model_name?.trim()) count++;
  if (c.sampler?.trim()) count++;
  if (c.min_steps != null || c.max_steps != null) count++;
  if (c.min_cfg != null || c.max_cfg != null) count++;
  if (c.min_rating != null || c.max_rating != null) count++;
  if (c.min_aesthetic != null || c.max_aesthetic != null) count++;
  if (c.is_favorite != null) count++;
  if (c.is_nsfw != null) count++;
  if (c.media_type) count++;
  if (c.min_duration != null || c.max_duration != null) count++;
  if (c.min_fps != null || c.max_fps != null) count++;
  return count;
}

/**
 * Tokenize input respecting single and double quotes.
 */
function tokenizeQuery(input: string): string[] {
  const tokens: string[] = [];
  let current = "";
  let inQuotes = false;
  let quoteChar = '"';

  for (let i = 0; i < input.length; i++) {
    const ch = input[i];
    if (inQuotes) {
      if (ch === quoteChar) {
        inQuotes = false;
      } else {
        current += ch;
      }
    } else if (ch === '"' || ch === "'") {
      inQuotes = true;
      quoteChar = ch;
    } else if (/\s/.test(ch)) {
      if (current.length > 0) {
        tokens.push(current);
        current = "";
      }
    } else {
      current += ch;
    }
  }
  if (current.length > 0) {
    tokens.push(current);
  }
  return tokens;
}

function parseRange(val: string): [number | null, number | null] {
  if (val.includes("..")) {
    const [start, end] = val.split("..", 2);
    const min = start ? Number(start.trim()) : null;
    const max = end ? Number(end.trim()) : null;
    return [Number.isNaN(min) ? null : min, Number.isNaN(max) ? null : max];
  }
  if (val.includes("-") && !val.startsWith("-")) {
    const parts = val.split("-");
    if (parts.length === 2 && parts[0] && parts[1]) {
      const min = Number(parts[0].trim());
      const max = Number(parts[1].trim());
      return [Number.isNaN(min) ? null : min, Number.isNaN(max) ? null : max];
    }
  }
  if (val.startsWith(">=")) {
    const num = Number(val.slice(2).trim());
    return [Number.isNaN(num) ? null : num, null];
  }
  if (val.startsWith(">")) {
    const num = Number(val.slice(1).trim());
    return [Number.isNaN(num) ? null : num, null];
  }
  if (val.startsWith("<=")) {
    const num = Number(val.slice(2).trim());
    return [null, Number.isNaN(num) ? null : num];
  }
  if (val.startsWith("<")) {
    const num = Number(val.slice(1).trim());
    return [null, Number.isNaN(num) ? null : num];
  }
  if (val.startsWith("=")) {
    const num = Number(val.slice(1).trim());
    return [Number.isNaN(num) ? null : num, Number.isNaN(num) ? null : num];
  }
  const exact = Number(val.trim());
  return [Number.isNaN(exact) ? null : exact, Number.isNaN(exact) ? null : exact];
}

/**
 * Parse a user query string into structured SearchCriteria and bare text terms.
 */
export function parseSearchQuery(query: string): SearchCriteria {
  const criteria: SearchCriteria = {};
  if (!query || !query.trim()) return criteria;

  const rawTokens = tokenizeQuery(query);
  const tokens: string[] = [];
  let idx = 0;
  while (idx < rawTokens.length) {
    const tok = rawTokens[idx];
    if (tok.endsWith(":") && idx + 1 < rawTokens.length) {
      tokens.push(tok + rawTokens[idx + 1]);
      idx += 2;
    } else {
      tokens.push(tok);
      idx += 1;
    }
  }

  const bareTerms: string[] = [];

  for (const token of tokens) {
    const colonIdx = token.indexOf(":");
    if (colonIdx > 0) {
      const key = token.slice(0, colonIdx).trim().toLowerCase();
      const val = token.slice(colonIdx + 1).trim();
      if (!val) continue;

      switch (key) {
        case "prompt":
          criteria.prompt = val;
          break;
        case "neg":
        case "negative":
        case "negative_prompt":
          criteria.negative_prompt = val;
          break;
        case "model":
        case "model_name":
          criteria.model_name = val;
          break;
        case "hash":
        case "model_hash":
          criteria.model_hash = val;
          break;
        case "sampler":
          criteria.sampler = val;
          break;
        case "steps": {
          const [min, max] = parseRange(val);
          criteria.min_steps = min;
          criteria.max_steps = max;
          break;
        }
        case "cfg":
        case "cfg_scale": {
          const [min, max] = parseRange(val);
          criteria.min_cfg = min;
          criteria.max_cfg = max;
          break;
        }
        case "rating": {
          const [min, max] = parseRange(val);
          criteria.min_rating = min;
          criteria.max_rating = max;
          break;
        }
        case "aesthetic":
        case "aesthetic_score": {
          const [min, max] = parseRange(val);
          criteria.min_aesthetic = min;
          criteria.max_aesthetic = max;
          break;
        }
        case "fav":
        case "favorite": {
          const lower = val.toLowerCase();
          if (["true", "yes", "1"].includes(lower)) criteria.is_favorite = true;
          else if (["false", "no", "0"].includes(lower)) criteria.is_favorite = false;
          break;
        }
        case "nsfw": {
          const lower = val.toLowerCase();
          if (["true", "yes", "1"].includes(lower)) criteria.is_nsfw = true;
          else if (["false", "no", "0"].includes(lower)) criteria.is_nsfw = false;
          break;
        }
        case "is": {
          const lower = val.toLowerCase();
          if (["fav", "favorite"].includes(lower)) criteria.is_favorite = true;
          else if (lower === "nsfw") criteria.is_nsfw = true;
          else if (lower === "sfw") criteria.is_nsfw = false;
          break;
        }
        case "type":
          criteria.media_type = val;
          break;
        case "duration": {
          const [min, max] = parseRange(val);
          criteria.min_duration = min;
          criteria.max_duration = max;
          break;
        }
        case "fps": {
          const [min, max] = parseRange(val);
          criteria.min_fps = min;
          criteria.max_fps = max;
          break;
        }
        default:
          bareTerms.push(token);
          break;
      }
    } else {
      bareTerms.push(token);
    }
  }

  if (bareTerms.length > 0) {
    criteria.text = bareTerms.join(" ");
  }

  return criteria;
}
