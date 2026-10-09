#!/usr/bin/env python3
"""Tests for farm_alerts.py. Run: python3 -m unittest test_farm_alerts -v"""

import unittest

import farm_alerts

# The sample plan of BACKEND.md 2.4, word for word.
SAMPLE_PLAN = {
    "from": "2026-10-08",
    "days": 10,
    "rain_mm": [0, 0, 2.1, 14.2, 0, 0, 0, 0, 0, 0],
    "tmin": [9, 8, 7, -3, 6, 7, 8, 9, 9, 10],
    "tmax": [24, 23, 20, 15, 19, 22, 24, 25, 25, 26],
    "alerts": [
        {"type": "frost", "day": "2026-10-11", "value": -3, "level": "alarm", "ku": "...", "en": "Frost -3 °C Sun night"}
    ],
    "decisions": [
        {"code": "frost_check", "ku": "...", "en": "Hard frost on Sun. Check the heads 7 to 10 days after."}
    ],
    "source": "Open-Meteo (ECMWF/GraphCast family)",
    "issued": "2026-10-08T06:00:00Z",
}

FARM = {"id": "7", "lat": 36.0, "lon": 44.0}


def fire(lat, lon, status="active"):
    return {"id": "31", "lat": lat, "lon": lon, "detected_at": "2026-10-09T11:20:00Z", "status": status}


class PlanAlerts(unittest.TestCase):
    def test_the_sample_plan_gives_one_frost_alarm_keyed_by_type_and_day(self):
        alerts = farm_alerts.alerts_from_plan(SAMPLE_PLAN)

        self.assertEqual(len(alerts), 1)
        key, body = alerts[0]
        self.assertEqual(key, "frost:2026-10-11")
        self.assertEqual(body["type"], "frost")
        self.assertEqual(body["day"], "2026-10-11")
        self.assertEqual(body["level"], "alarm")
        self.assertEqual(body["confidence"], "likely")
        self.assertEqual(body["ku"], "...", "the Sorani is the plan's own, never written here")
        self.assertEqual(body["en"], "Frost -3 °C Sun night")
        self.assertEqual(body["action_en"], "Hard frost on Sun. Check the heads 7 to 10 days after.")
        self.assertEqual(body["action_ku"], "...")

    def test_an_alert_without_a_matching_decision_repeats_its_own_text_as_the_action(self):
        plan = dict(SAMPLE_PLAN, decisions=[])

        _, body = farm_alerts.alerts_from_plan(plan)[0]

        self.assertEqual(body["action_en"], body["en"])
        self.assertEqual(body["action_ku"], body["ku"])

    def test_an_unknown_type_or_level_or_a_missing_text_is_left_out(self):
        plan = {
            "alerts": [
                {"type": "locusts", "day": "2026-10-11", "level": "alarm", "ku": "a", "en": "a"},
                {"type": "heat", "day": "2026-10-11", "level": "red", "ku": "a", "en": "a"},
                {"type": "heat", "day": "2026-10-11", "level": "watch", "ku": "", "en": "a"},
                {"type": "heat", "day": "soon", "level": "watch", "ku": "a", "en": "a"},
            ]
        }

        self.assertEqual(farm_alerts.alerts_from_plan(plan), [])

    def test_a_plan_without_alerts_gives_none(self):
        self.assertEqual(farm_alerts.alerts_from_plan({"from": "2026-10-08"}), [])

    def test_two_alerts_of_one_type_and_day_give_one_key_and_the_alarm_wins(self):
        plan = {
            "alerts": [
                {"type": "heat", "day": "2026-10-11", "level": "watch", "ku": "a", "en": "warm"},
                {"type": "heat", "day": "2026-10-11", "level": "alarm", "ku": "b", "en": "hot"},
            ]
        }

        alerts = farm_alerts.alerts_from_plan(plan)

        self.assertEqual([key for key, _ in alerts], ["heat:2026-10-11"])
        self.assertEqual(alerts[0][1]["en"], "hot")


class FireAlerts(unittest.TestCase):
    def test_a_fire_within_two_km_is_an_alarm_that_never_claims_a_confirmed_fire(self):
        key, body = farm_alerts.fire_alert(FARM, fire(36.009, 44.011))

        self.assertEqual(key, "fire:31")
        self.assertEqual(body["level"], "alarm")
        self.assertEqual(body["confidence"], "unsure")
        self.assertIn("Satellite fire detection", body["en"])
        self.assertIn("not been checked", body["en"])
        self.assertIn("north-east", body["en"])
        self.assertIn("north-east", body["action_en"])
        self.assertEqual(body["day"], "2026-10-09")
        self.assertIn("14:20", body["en"], "the time is Iraq's, three hours ahead of UTC")

    def test_a_fire_between_two_and_five_km_is_a_watch(self):
        _, body = farm_alerts.fire_alert(FARM, fire(35.97, 44.0))

        self.assertEqual(body["level"], "watch")
        self.assertIn("3.3 km to the south", body["en"])

    def test_a_fire_beyond_five_km_gives_no_alert(self):
        self.assertIsNone(farm_alerts.fire_alert(FARM, fire(36.06, 44.0)))

    def test_a_fire_that_is_out_gives_no_alert(self):
        self.assertEqual(farm_alerts.alerts_from_fires(FARM, [fire(36.009, 44.011, "out")]), [])

    def test_the_eight_directions(self):
        self.assertEqual(farm_alerts.direction(36, 44, 37, 44), "north")
        self.assertEqual(farm_alerts.direction(36, 44, 36, 45), "east")
        self.assertEqual(farm_alerts.direction(36, 44, 35, 44), "south")
        self.assertEqual(farm_alerts.direction(36, 44, 36, 43), "west")
        self.assertEqual(farm_alerts.direction(36, 44, 35, 43), "south-west")


if __name__ == "__main__":
    unittest.main()
