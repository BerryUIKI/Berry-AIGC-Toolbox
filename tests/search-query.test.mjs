import assert from "node:assert/strict";
import test from "node:test";
import { criteriaToQuery, parseSearchQuery } from "../src/utils/search.ts";

test("serializes model hashes without dropping the structured filter", () => {
  assert.equal(criteriaToQuery({ model_hash: "abc12345" }), "hash:abc12345");
  assert.equal(criteriaToQuery({ model_hash: "hash with spaces" }), 'hash:"hash with spaces"');
});

test("serializes video search criteria correctly", () => {
  assert.equal(criteriaToQuery({ media_type: "video" }), "type:video");
  assert.equal(criteriaToQuery({ min_duration: 5, max_duration: 15 }), "duration:5..15");
  assert.equal(criteriaToQuery({ min_duration: 10 }), "duration:>=10");
  assert.equal(criteriaToQuery({ max_duration: 60 }), "duration:<=60");
  assert.equal(criteriaToQuery({ min_fps: 30, max_fps: 60 }), "fps:30..60");
  assert.equal(criteriaToQuery({ min_fps: 24 }), "fps:>=24");
});

test("parses structured tokens and freeform text via parseSearchQuery", () => {
  const criteria = parseSearchQuery('cyberpunk rainy street model:"SDXL Turbo" rating:>=5 fav:true steps:20..40');
  assert.equal(criteria.text, "cyberpunk rainy street");
  assert.equal(criteria.model_name, "SDXL Turbo");
  assert.equal(criteria.min_rating, 5);
  assert.equal(criteria.is_favorite, true);
  assert.equal(criteria.min_steps, 20);
  assert.equal(criteria.max_steps, 40);
});

test("criteriaToQuery preserves freeform text keyword when present", () => {
  const query = criteriaToQuery({
    text: "cyberpunk rainy street",
    model_name: "SDXL",
    min_rating: 4,
  });
  assert.ok(query.includes("cyberpunk rainy street"));
  assert.ok(query.includes("model:SDXL"));
  assert.ok(query.includes("rating:>=4"));
});

