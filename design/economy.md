# Economy Design

Status: **Draft** — not yet implemented

---

## Goals

- Gold should feel meaningful: hard enough to earn that spending it matters
- Player vendor stands create a passive economy that runs while players are offline
- Auctions create excitement and price discovery for rare items
- Shops provide a floor price so the economy doesn't completely collapse

---

## Currency

Single currency: **credits** (cr).

| Store | Location | Notes |
|-------|----------|-------|
| Carried credits | Credit chip in player inventory | Lost on death (chip drops on corpse) |
| Bank balance | Per-character persistent | Safe, accessible at banking terminal fixtures only |

Credits are the Colonial Development Authority's official currency — digital, Corporate-tracked, used at all official vendors.

**Barter:** Players can also trade value-for-value without credits — swapping items directly. The economy supports both. Shops and official vendors require credits; player-to-player trades can be credits, goods, or a mix. Item `value` fields in templates give a credit reference price for barter negotiation, not a hard exchange rate.

---

## Shops (NPC Vendors)

Defined in zone files on a mob or room.

```json
"shop": {
  "buy_markup": 1.5,
  "sell_markdown": 0.4,
  "inventory": ["item_id_1", "item_id_2"]
}
```

Commands: `list`, `buy <item>`, `sell <item>`, `value <item>`

- Player sells to shop: receives `item_base_value × sell_markdown`
- Player buys from shop: pays `item_base_value × buy_markup`
- Shop inventory is infinite (static vendors don't run out)

---

## Player Vendor Stands

A player rents a stand in a designated market Area or Room. They stock it with items and set prices. Items sell to other players while the owner is offline.

### Flow

1. Player pays a daily rent fee to activate their stand (`rent stand`)
2. Player adds items: `stock <item> <price>`
3. Other players can browse: `browse stands`, `browse <player>`
4. Purchase: `buy <item> from <player>` — credits transferred immediately, item goes to buyer inventory
5. Earnings accumulate in a stand ledger, collected with `collect earnings`
6. Stand expires if rent not paid — items returned to owner's mailbox

### Open questions

- Where is the market room? One per zone? One global market?
- Max items per stand?
- What happens to items if a character is deleted?

---

## Auction System

Any player can auction an item. Bidding is timed.

### Flow

1. `auction <item> <min-bid> [duration in minutes]` — item held in escrow, announcement broadcast
2. Other players bid: `bid <amount>` (must be in same zone? or global?)
3. Timer counts down with periodic announcements (5 min, 1 min, going once/twice/sold)
4. Winner: item transferred, credits deducted
5. Outbid players: credits immediately returned
6. No bids: item returned to auctioneer

### Open questions

- Global auction or zone-local?
- Can you auction from anywhere or only at an auction house room?
- Minimum bid increment (flat or percentage)?
- Cancel mechanism — can the auctioneer cancel, and at what cost?

---

## Loot Economy Balance (notes for later)

- Mob credits drops should be the primary income source for new characters
- Rare items should have low drop rates to maintain auction value
- Vendor markdown (40%) creates a meaningful floor without being a credits sink
- Death (dropping carried credits) is the primary credits sink; banking mitigates but doesn't eliminate it
