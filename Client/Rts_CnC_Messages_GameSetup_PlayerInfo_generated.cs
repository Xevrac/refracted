using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_PlayerInfo
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.GameSetup.PlayerInfo); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.GameSetup.PlayerInfo)obj;
            //  Serialize PlayerID
            s.Write(value.PlayerID);
            //  Serialize Reconnect
            s.Write(value.Reconnect);
            //  Serialize Faction
            s.Write(value.Faction);
            //  Serialize GeneralId
            s.Write(value.GeneralId);
            //  Serialize Team
            s.Write(value.Team);
            //  Serialize StartPoint
            s.Write(value.StartPoint);
            //  Serialize Difficulty
            s.Write(value.Difficulty);
            //  Serialize IsAI
            s.Write(value.IsAI);
            //  Serialize array AllegianceLevels
            Rts.Serialization.Reference.Write(s, value.AllegianceLevels, () =>
            {
                s.WriteVarInt32(value.AllegianceLevels.Length);
                for(int i = 0 ; i < value.AllegianceLevels.Length ; ++i)
                {
                    s.Write(value.AllegianceLevels[i]);
                }
            });
            //  Serialize array SkillTreeUnlocks
            Rts.Serialization.Reference.Write(s, value.SkillTreeUnlocks, () =>
            {
                s.WriteVarInt32(value.SkillTreeUnlocks.Length);
                for(int i = 0 ; i < value.SkillTreeUnlocks.Length ; ++i)
                {
                    s.Write(value.SkillTreeUnlocks[i]);
                }
            });
            //  Serialize ConsumablePlayerPower
            s.Write(value.ConsumablePlayerPower);
            //  Serialize EnableSkillTree
            s.Write(value.EnableSkillTree);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            Rts.CnC.Messages.GameSetup.PlayerInfo value = default(Rts.CnC.Messages.GameSetup.PlayerInfo);
            DeserializeValue(s, ref value);
            return value;
        }
        
        public static void DeserializeValue(System.IO.Stream s, ref Rts.CnC.Messages.GameSetup.PlayerInfo value)
        {
            var valueRef = __makeref(value);
            //  Deserialize PlayerID
            s.Read(out value.PlayerID);
            //  Deserialize Reconnect
            s.Read(out value.Reconnect);
            //  Deserialize Faction
            s.Read(out value.Faction);
            //  Deserialize GeneralId
            s.Read(out value.GeneralId);
            //  Deserialize Team
            s.Read(out value.Team);
            //  Deserialize StartPoint
            s.Read(out value.StartPoint);
            //  Deserialize Difficulty
            s.Read(out value.Difficulty);
            //  Deserialize IsAI
            s.Read(out value.IsAI);
            //  Deserialize array AllegianceLevels
            Rts.Serialization.Reference.Read(s, out value.AllegianceLevels, () =>
            {
                int length = s.ReadVarInt32();
                System.Single[] tmp = new System.Single[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize array SkillTreeUnlocks
            Rts.Serialization.Reference.Read(s, out value.SkillTreeUnlocks, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize ConsumablePlayerPower
            s.Read(out value.ConsumablePlayerPower);
            //  Deserialize EnableSkillTree
            s.Read(out value.EnableSkillTree);

        }
    }
}
