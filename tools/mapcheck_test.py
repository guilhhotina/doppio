import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mapcheck as mc

CORRIDOR = [
    "##########",
    "#........#",
    "####D#####",
    "#........#",
    "##########",
]


def grid(rows, fog_seen=10.0):
    m = mc.Map()
    m.w, m.h = len(rows[0]), len(rows)
    m.solid = rows
    m.tile = lambda x, y: rows[y][x] if 0 <= x < m.w and 0 <= y < m.h else " "
    m.walk = {(x, y) for y in range(m.h) for x in range(m.w) if rows[y][x] in mc.WALK}
    m.doors = {(x, y): {"axis": "h"} for y in range(m.h) for x in range(m.w) if rows[y][x] == "D"}
    m.portals = {}
    m.fog_seen = fog_seen
    return m


def cam(eye, target, fov=60.0):
    return {"rect": [0, 0, 0, 0], "v": list(eye) + list(target) + [fov], "line": 0}


class Project(unittest.TestCase):
    def test_centre_and_edges(self):
        v = [0.0, 1.0, 0.0, 0.0, 1.0, 5.0, 60.0]
        self.assertEqual(mc.project(v, (0.0, 1.0, 3.0)), (0.0, 0.0))
        self.assertGreater(abs(mc.project(v, (6.0, 1.0, 3.0))[0]), 1.0)
        self.assertIsNone(mc.project(v, (0.0, 1.0, -1.0)))


class Body(unittest.TestCase):
    def test_player_between_eye_and_point_hides_it(self):
        v = [0.5, 1.2, 0.5, 4.5, 1.0, 0.5, 60.0]
        self.assertTrue(mc.body_blocks(v, (4.5, 1.0, 0.5), (2, 0)))
        self.assertFalse(mc.body_blocks(v, (4.5, 1.0, 0.5), (2, 1)))

    def test_ray_over_the_head_is_clear(self):
        v = [0.5, 2.4, 0.5, 4.5, 2.4, 0.5, 60.0]
        self.assertFalse(mc.body_blocks(v, (4.5, 2.4, 0.5), (2, 0)))


class Door(unittest.TestCase):
    def setUp(self):
        self.m = grid(CORRIDOR)
        _, self.seen = mc.door_thing(self.m, (4, 2))

    def test_seen_from_across_the_corridor(self):
        self.assertTrue(self.seen(cam((4.5, 2.2, 1.2), (4.5, 1.0, 3.0)), None))
        self.assertTrue(self.seen(cam((5.3, 2.2, 1.2), (4.5, 1.0, 2.5)), None))

    def test_hidden_by_the_jamb_down_a_long_corridor(self):
        self.assertFalse(self.seen(cam((8.5, 2.2, 1.5), (1.0, 1.0, 1.6)), None))

    def test_needs_the_frustum_and_the_fog(self):
        self.assertFalse(self.seen(cam((4.5, 2.2, 1.2), (4.5, 1.0, -3.0)), None))
        self.m.fog_seen = 1.0
        self.assertFalse(self.seen(cam((4.5, 2.2, 1.2), (4.5, 1.0, 3.0)), None))

    def test_hidden_by_the_player_in_front_of_it(self):
        self.assertFalse(self.seen(cam((4.5, 1.2, 1.2), (4.5, 1.2, 3.0), 40.0), (4, 1)))


class Points(unittest.TestCase):
    def test_wall_needs_los(self):
        m = grid(CORRIDOR)
        _, seen = mc.point_thing(m, (2, 3), [(2.5, 0.05, 3.5)])
        self.assertFalse(seen(cam((2.5, 2.2, 1.2), (2.5, 0.0, 3.5)), None))
        self.assertTrue(seen(cam((6.5, 2.2, 3.5), (2.5, 0.0, 3.5)), None))


if __name__ == "__main__":
    unittest.main()
