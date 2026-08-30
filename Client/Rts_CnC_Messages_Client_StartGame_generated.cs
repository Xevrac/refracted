using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_StartGame
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.StartGame); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.StartGame)obj;
            //  Serialize Faction
            s.Write(value.Faction);
            //  Serialize array PlayerIds
            Rts.Serialization.Reference.Write(s, value.PlayerIds, () =>
            {
                s.WriteVarInt32(value.PlayerIds.Length);
                for(int i = 0 ; i < value.PlayerIds.Length ; ++i)
                {
                    s.Write(value.PlayerIds[i]);
                }
            });
            //  Serialize array PlayerTypes
            Rts.Serialization.Reference.Write(s, value.PlayerTypes, () =>
            {
                s.WriteVarInt32(value.PlayerTypes.Length);
                for(int i = 0 ; i < value.PlayerTypes.Length ; ++i)
                {
                    s.Write(value.PlayerTypes[i]);
                }
            });
            //  Serialize array AllegianceLevels
            Rts.Serialization.Reference.Write(s, value.AllegianceLevels, () =>
            {
                s.WriteVarInt32(value.AllegianceLevels.Length);
                for(int i = 0 ; i < value.AllegianceLevels.Length ; ++i)
                {
                    s.Write(value.AllegianceLevels[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.StartGame)) as Rts.CnC.Messages.Client.StartGame;
            //  Deserialize Faction
            s.Read(out value.Faction);
            //  Deserialize array PlayerIds
            Rts.Serialization.Reference.Read(s, out value.PlayerIds, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize array PlayerTypes
            Rts.Serialization.Reference.Read(s, out value.PlayerTypes, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
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

            return value;
        }
        
    }
}
