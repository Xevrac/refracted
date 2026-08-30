using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PlayerAllegianceChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PlayerAllegianceChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PlayerAllegianceChanged)obj;
            //  Serialize array PlayerIds
            Rts.Serialization.Reference.Write(s, value.PlayerIds, () =>
            {
                s.WriteVarInt32(value.PlayerIds.Length);
                for(int i = 0 ; i < value.PlayerIds.Length ; ++i)
                {
                    s.Write(value.PlayerIds[i]);
                }
            });
            //  Serialize array PlayerAllegianceLevels
            Rts.Serialization.Reference.Write(s, value.PlayerAllegianceLevels, () =>
            {
                s.WriteVarInt32(value.PlayerAllegianceLevels.Length);
                for(int i = 0 ; i < value.PlayerAllegianceLevels.Length ; ++i)
                {
                    s.Write(value.PlayerAllegianceLevels[i]);
                }
            });
            //  Serialize AllegianceLevelsPerPlayer
            s.Write(value.AllegianceLevelsPerPlayer);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PlayerAllegianceChanged)) as Rts.CnC.Messages.Client.PlayerAllegianceChanged;
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
            //  Deserialize array PlayerAllegianceLevels
            Rts.Serialization.Reference.Read(s, out value.PlayerAllegianceLevels, () =>
            {
                int length = s.ReadVarInt32();
                System.Single[] tmp = new System.Single[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize AllegianceLevelsPerPlayer
            s.Read(out value.AllegianceLevelsPerPlayer);

            return value;
        }
        
    }
}
