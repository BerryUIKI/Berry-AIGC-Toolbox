import assert from "node:assert/strict";
import test from "node:test";
import { normalizeTableColumns } from "../src/utils/table-columns.ts";

test("legacy and malformed Table preferences resolve to independent compact defaults", () => {
  const first = normalizeTableColumns(undefined);
  assert.deepEqual(first.filter(c=>c.visible).map(c=>c.id), ['preview','name','prompt','model']);
  assert.ok(first.filter(c=>c.visible).reduce((sum,c)=>sum+c.width,28)<=460);
  first[0].width=600;
  assert.equal(normalizeTableColumns([])[0].width,44);
  assert.deepEqual(normalizeTableColumns('invalid'),normalizeTableColumns([]));
});

test("Table preference normalization keeps required name, ignores unknown IDs and bounds widths", () => {
  const columns = normalizeTableColumns([{id:'name',visible:false,width:-10},{id:'prompt',visible:true,width:1000},
    {id:'model',visible:false,width:NaN},{id:'unknown',visible:true,width:123},{id:'name',visible:true,width:600}]);
  assert.deepEqual(columns.find(c=>c.id==='name'),{id:'name',visible:true,width:80});
  assert.equal(columns.find(c=>c.id==='prompt').width,640);
  assert.deepEqual(columns.find(c=>c.id==='model'),{id:'model',visible:false,width:100});
  assert.equal(columns.length,9);
  assert.deepEqual(normalizeTableColumns(JSON.parse(JSON.stringify(columns))),columns);
});
