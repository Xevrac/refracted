using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AbilityActivated_EUAirRaid
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.AbilityActivated_EUAirRaid); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.AbilityActivated_EUAirRaid)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize array UnitIds
            Rts.Serialization.Reference.Write(s, value.UnitIds, () =>
            {
                s.WriteVarInt32(value.UnitIds.Length);
                for(int i = 0 ; i < value.UnitIds.Length ; ++i)
                {
                    s.Write(value.UnitIds[i]);
                }
            });
            //  Serialize AbilityId
            s.Write(value.AbilityId);
            //  Serialize PlayerPowerPosition
            s.Write(value.PlayerPowerPosition);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.AbilityActivated_EUAirRaid)) as Rts.CnC.Messages.Client.AbilityActivated_EUAirRaid;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize array UnitIds
            Rts.Serialization.Reference.Read(s, out value.UnitIds, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);
            //  Deserialize PlayerPowerPosition
            s.Read(out value.PlayerPowerPosition);

            return value;
        }
        
    }
}
