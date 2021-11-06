// Vendored and adjusted for clang and Rust
// Original: https://github.com/blynn/dlx

// See http://en.wikipedia.org/wiki/Dancing_Links.
#include <limits.h>
#include <stdlib.h>
#include "dlx.h"

#define F(i,n) for(int i = 0; i < n; i++)

#define C(i,n,dir) for(cell_ptr i = (n)->dir; i != n; i = i->dir)

struct cell_s;
typedef struct cell_s *cell_ptr;
struct cell_s {
  cell_ptr U, D, L, R;
  int n;
  union {
    cell_ptr c;
    int s;
  };
};

struct dlx_s {
  int ctabn, rtabn, ctab_alloc, rtab_alloc;
  cell_ptr *ctab, *rtab;
  cell_ptr root;
};

// Some link dance moves.
static cell_ptr LR_self(cell_ptr c) {
  return c->L = c->R = c;
}
static cell_ptr UD_self(cell_ptr c) {
  return c->U = c->D = c;
}

// Undeletable deletes.
static cell_ptr LR_delete(cell_ptr c) {
  return c->L->R = c->R, c->R->L = c->L, c;
}
static cell_ptr UD_delete(cell_ptr c) {
  return c->U->D = c->D, c->D->U = c->U, c;
}

// Undelete.
static cell_ptr UD_restore(cell_ptr c) {
  return c->U->D = c->D->U = c;
}
static cell_ptr LR_restore(cell_ptr c) {
  return c->L->R = c->R->L = c;
}

// Insert cell j to the left of cell k.
static cell_ptr LR_insert(cell_ptr j, cell_ptr k) {
  return j->L = k->L, j->R = k, k->L = k->L->R = j;
}

// Insert cell j above cell k.
static cell_ptr UD_insert (cell_ptr j, cell_ptr k) {
  return j->U = k->U, j->D = k, k->U = k->U->D = j;
}

static cell_ptr col_new() {
  cell_ptr c = malloc(sizeof(*c));
  UD_self(c)->s = 0;
  return c;
}

dlx_t dlx_new() {
  dlx_t p = malloc(sizeof(*p));
  p->ctabn = p->rtabn = 0;
  p->ctab_alloc = p->rtab_alloc = 8;
  p->ctab = malloc(sizeof(cell_ptr) * p->ctab_alloc);
  p->rtab = malloc(sizeof(cell_ptr) * p->rtab_alloc);
  p->root = LR_self(col_new());
  return p;
}

void dlx_clear(dlx_t p) {
  // Elements in the LR list for each row are never covered, thus all cells
  // can be accessed from the 'rtab' LR lists.
  F(i, p->rtabn) {
    cell_ptr r = p->rtab[i];
    if (r) {
      cell_ptr next;
      for(cell_ptr j = r->R; j != r; j = next) {
        next = j->R;
        free(j);
      }
      free(r);
    }
  }
  // Columns may be covered, but they are always accessible from 'ctab'.
  F(i, p->ctabn) {
    free(p->ctab[i]);
  }
  free(p->rtab);
  free(p->ctab);
  free(p->root);
  free(p);
}

int dlx_rows(dlx_t dlx) {
  return dlx->rtabn;
}

int dlx_cols(dlx_t dlx) {
  return dlx->ctabn;
}

static void dlx_add_col(dlx_t p) {
  cell_ptr c = col_new();
  LR_insert(c, p->root);
  c->n = p->ctabn++;
  if (p->ctabn == p->ctab_alloc) {
    p->ctab = realloc(p->ctab, sizeof(cell_ptr) * (p->ctab_alloc *= 2));
  }
  p->ctab[c->n] = c;
}

static void dlx_add_row(dlx_t p) {
  if (p->rtabn == p->rtab_alloc) {
    p->rtab = realloc(p->rtab, sizeof(cell_ptr) * (p->rtab_alloc *= 2));
  }
  p->rtab[p->rtabn++] = 0;
}

static void alloc_col(dlx_t p, int n) {
  while(p->ctabn <= n) {
    dlx_add_col(p);
  }
}

static void alloc_row(dlx_t p, int n) {
  while(p->rtabn <= n) {
    dlx_add_row(p);
  }
}

void dlx_mark_optional(dlx_t p, int col) {
  alloc_col(p, col);
  cell_ptr c = p->ctab[col];
  // Prevent undeletion by self-linking.
  LR_self(LR_delete(c));
}

static cell_ptr dlx_set_new(int row, cell_ptr c) {
  cell_ptr n = malloc(sizeof(*n));
  n->n = row;
  n->c = c;
  c->s++;
  UD_insert(n, c);
  return n;
}

void dlx_set(dlx_t p, int row, int col) {
  // We don't bother sorting. DLX works fine with jumbled rows and columns.
  // We just have to watch out for duplicates. (Actually, I think the DLX code
  // works even with duplicates, though it would be inefficient.)
  //
  // For a given column, the UD list is ordered in the order that dlx_set()
  // is called, not by row number. Similarly for a given row and its LR list.
  alloc_row(p, row);
  alloc_col(p, col);
  cell_ptr c = p->ctab[col];
  cell_ptr *rp = p->rtab + row;
  if (!*rp) {
    *rp = LR_self(dlx_set_new(row, c));
    return;
  }
  // Ignore duplicates.
  if ((*rp)->c->n == col) {
    return;
  }
  C(r, *rp, R) {
    if (r->c->n == col) {
      return;
    }
  }
  // Otherwise insert at end of LR list.
  LR_insert(dlx_set_new(row, c), *rp);
}

static void cover_col(cell_ptr c) {
  LR_delete(c);
  C(i, c, D) {
    C(j, i, R) {
      UD_delete(j)->c->s--;
    }
  }
}

static void uncover_col(cell_ptr c) {
  C(i, c, U) {
    C(j, i, L) {
      UD_restore(j)->c->s++;
    }
  }
  LR_restore(c);
}

int dlx_pick_row(dlx_t p, int i) {
  if (i < 0 || i >= p->rtabn) {
    return -1;
  }
  cell_ptr r = p->rtab[i];
  if (!r) {
    return 0;  // Empty row.
  }
  cover_col(r->c);
  C(j, r, R) {
    cover_col(j->c);
  }
  return 0;
}

int dlx_remove_row(dlx_t p, int i) {
  if (i < 0 || i >= p->rtabn) {
    return -1;
  }
  cell_ptr r = p->rtab[i];
  if (!r) {
    return 0;  // Empty row.
  }
  UD_delete(r)->c->s--;
  C(j, r, R) {
    UD_delete(j)->c->s--;
  }
  p->rtab[i] = 0;
  return 0;
}

static int dlx_solve_inner(dlx_t p,
                           int greedy,
                           void (*cover_cb)(void*, int, int, int),
                           void (*uncover_cb)(void*),
                           void (*found_cb)(void*),
                           void (*stuck_cb)(void*, int, int),
                           void *data,
                           int depth) {
  cell_ptr c = p->root->R;
  int rc = -1;
  if (c == p->root) {
    if (found_cb) {
      found_cb(data);
    }
    return 0;
  }
  int s = INT_MAX;  // S-heuristic: choose first most-constrained column.
  C(i, p->root, R) {
    if (i->s < s) {
      s = (c = i)->s;
    }
  }
  if (!s) {
    if (stuck_cb) {
      stuck_cb(data, c->n, depth);
    }
    return -1;
  }
  cover_col(c);
  C(r, c, D) {
    if (cover_cb) {
      cover_cb(data, c->n, s, r->n);
    }
    C(j, r, R) {
      cover_col(j->c);
    }
    rc = dlx_solve_inner(
      p,
      greedy,
      cover_cb,
      uncover_cb,
      found_cb,
      stuck_cb,
      data,
      depth + 1
    );
    if (uncover_cb) {
      uncover_cb(data);
    }
    C(j, r, L) {
      uncover_col(j->c);
    }
    if (!greedy && rc == 0) {
      break;
    }
  }
  uncover_col(c);
  return rc;
}

int dlx_solve(dlx_t p,
              int greedy,
              void (*cover_cb)(void*, int, int, int),
              void (*uncover_cb)(void*),
              void (*found_cb)(void*),
              void (*stuck_cb)(void*, int, int),
              void *data) {
  return dlx_solve_inner(
    p,
    greedy,
    cover_cb,
    uncover_cb,
    found_cb,
    stuck_cb,
    data,
    0
  );
}

typedef struct dlx_solve_cb_locals {
  int soln;
  int *sol;
  void (*cb)(void*, int[], int);
  void *data;
} dlx_solve_cb_locals;

static void dlx_solve_cover_cb(void *data, int _c, int _s, int r) {
  dlx_solve_cb_locals *locals = (dlx_solve_cb_locals*) data;
  locals->sol[locals->soln++] = r;
}

static void dlx_solve_uncover_cb(void *data) {
  dlx_solve_cb_locals *locals = (dlx_solve_cb_locals*) data;
  locals->soln--;
}

static void dlx_solve_found_cb(void *data) {
  dlx_solve_cb_locals *locals = (dlx_solve_cb_locals*) data;
  locals->cb(locals->data, locals->sol, locals->soln);
}

static void dlx_solve_stuck_cb(void *_data, int _col, int _depth) {
}

void dlx_forall_cover(dlx_t p, void (*cb)(void*, int[], int), void *data) {
  int sol[p->rtabn];

  dlx_solve_cb_locals locals;
  locals.soln = 0;
  locals.sol = sol;
  locals.cb = cb;
  locals.data = data;

  dlx_solve(
    p,
    1,
    dlx_solve_cover_cb,
    dlx_solve_uncover_cb,
    dlx_solve_found_cb,
    dlx_solve_stuck_cb,
    &locals
  );
}

void dlx_first_cover(dlx_t p, void (*cb)(void*, int[], int), void *data) {
  int sol[p->rtabn];

  dlx_solve_cb_locals locals;
  locals.soln = 0;
  locals.sol = sol;
  locals.cb = cb;
  locals.data = data;

  dlx_solve(
    p,
    0,
    dlx_solve_cover_cb,
    dlx_solve_uncover_cb,
    dlx_solve_found_cb,
    dlx_solve_stuck_cb,
    &locals
  );
}
