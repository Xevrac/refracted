using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PotentialPlayerPowers
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PotentialPlayerPowers); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PotentialPlayerPowers)obj;
            //  Serialize array PlayerPowerIds
            Rts.Serialization.Reference.Write(s, value.PlayerPowerIds, () =>
            {
                s.WriteVarInt32(value.PlayerPowerIds.Length);
                for(int i = 0 ; i < value.PlayerPowerIds.Length ; ++i)
                {
                    s.Write(value.PlayerPowerIds[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PotentialPlayerPowers)) as Rts.CnC.Messages.Client.PotentialPlayerPowers;
            //  Deserialize array PlayerPowerIds
            Rts.Serialization.Reference.Read(s, out value.PlayerPowerIds, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
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
